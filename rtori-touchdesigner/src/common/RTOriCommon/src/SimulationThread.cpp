#include "Solver.hpp"

#include "rtori_core.hpp"

#include <chrono>
#include <cassert>
#include <algorithm>
#include <iostream>

#ifdef WIN32
#define NOMINMAX
#define WIN32_LEAN_AND_MEAN
#include <windows.h>
#include <winerror.h>
#include <processthreadsapi.h>
#endif

#ifdef UNIX
#include <pthread.h>
#endif

#include "rtori/td/SimulationThread.hpp"

constexpr const char* WORKER_THREAD_NAME = "RTOri TD SimulationThread";
constexpr const wchar_t* WORKER_THREAD_NAME_WIDE = L"RTOri TD SimulationThread";

using namespace rtori::rtori_td;

OutputGuard::OutputGuard(Output const& output, std::unique_lock<std::mutex>&& guard)
	: output(output), m_guard(std::move(guard)) {}

SimulationThread::SimulationThread(rtori::Context const* ctx) : m_ctx(ctx) {
	assert((void("ctx should be non-null"), ctx != nullptr));

#ifdef __cpp_lib_jthread
	this->m_threadHandler = std::jthread(&SimulationThread::runWorker, this);
#else
	this->m_threadHandler = std::thread(&SimulationThread::runWorker, this);
	this->m_stopRequestFlag = false;
#endif
}

SimulationThread::~SimulationThread() {
	this->requestStopWorker();
	if (this->m_threadHandler.joinable()) {
		this->m_threadHandler.join();
	}
}

Input const& SimulationThread::getInput() const {
	return this->m_input;
}

OutputGuard SimulationThread::getOutput() {
	std::unique_lock<std::mutex> guard(this->m_outputMutex);
	return OutputGuard(this->m_output, std::move(guard));
}

void SimulationThread::update(Input newInput) {
	std::lock_guard<std::mutex> lock(this->m_inputMutex);
	this->m_input = newInput;
}

void SimulationThread::notifyCook() {
	std::chrono::time_point<std::chrono::steady_clock> now = std::chrono::steady_clock::now();
	this->m_cookStart.store(now);
}

void SimulationThread::requestStopWorker() {
#ifdef __cpp_lib_jthread
	this->m_threadHandler.request_stop();
#else
	this->m_stopRequestFlag.store(true);
#endif
}

static void nameThread() {
#ifdef WIN32
	auto threadHandle = GetCurrentThread();
	HRESULT result = SetThreadDescription(threadHandle, WORKER_THREAD_NAME_WIDE);
	if (FAILED(result)) {
		// HAndle error better
		abort();
	}
#elif defined(MACOSX)
	pthread_setname_np(WORKER_THREAD_NAME);
#endif
}

std::string_view format_SolverOperationResult(rtori::SolverOperationResult result) {
	using rtori::SolverOperationResult;

	switch (result) {
	case SolverOperationResult::Success:
		return "Success";
	case SolverOperationResult::ErrorNotLoaded:
		return "Error(ErrorNotLoaded): Attempted to do an operation requiring that "
			   "a model be "
			   "loaded, and no model is";
	case SolverOperationResult::ErrorExtracting:
		return "Error(ErrorExtracting): Attempted to do an operation that can only "
			   "be done in "
			   "the 'Standby' or 'Loaded' state, while it was in the 'Extracting' "
			   "state";
	case SolverOperationResult::ErrorNoSuchFrameInFold:
		return "Error(ErrorNoSuchFrameInFold): No such Frame in fold";
	default:
		return "Error(Other): Other error";
	}
}

/// This is the inner state of the simulation thread
class SimulationThreadImpl final {
  public:
	SimulationThreadImpl(rtori::Context const* ctx) : ctx(ctx), solver(ctx) {
		nameThread();
	}

  private:
	rtori::Context const* ctx;
	bool extractPosition = false;
	bool extractVelocity = false;
	bool extractError = false;
	Solver solver;

	/// Tracks the last time a new input to the thread was given
	/// An input is normally triggered any time any change is made,
	/// so this doesn't mean it's a new FoldFile.
	int64_t lastInputNumber = -1;

	// Cook timing
	std::chrono::time_point<std::chrono::steady_clock> lastCookStart =
	  std::chrono::steady_clock::now();
	std::chrono::microseconds lastInterCookDuration = std::chrono::microseconds(0);
	bool packedThisFrame = false;
};

bool SimulationThread::isStopRequested() {
#ifdef __cpp_lib_jthread
	std::stop_token stopToken = this->m_threadHandler.get_stop_token();
	return stopToken.stop_requested();
#else
	return this->m_stopRequestFlag.load();
#endif 
}

void SimulationThread::runWorker() {
	assert((void("ctx should be non-null"), this->m_ctx != nullptr));

	if (isStopRequested()) {
		return;
	}

	nameThread();
	SimulationThreadImpl impl(this->m_ctx);

	bool extractPosition = false;
	bool extractVelocity = false;
	bool extractError = false;

	Solver solver(this->m_ctx);

	/// Tracks the last time a new input to the thread was given
	/// An input is normally triggered any time any change is made,
	/// so this doesn't mean it's a new FoldFile.
	int64_t lastInputNumber = -1;
	std::vector<float> verticesUnchanging;
	bool verticesCachedUnchanging = false;

	// Cook timing
	std::chrono::time_point<std::chrono::steady_clock> lastCookStart =
	  std::chrono::steady_clock::now();
	auto lastInterCookDuration = std::chrono::microseconds(0);
	bool packedThisFrame = false;

	bool loaded = false;

	// We keep the knowledge of how much time we spend stepping
	std::chrono::microseconds stepDuration = std::chrono::microseconds(std::chrono::seconds(1));

	while (!isStopRequested()) {
		// The inner loop
		bool newCook;
		{
			std::chrono::time_point<std::chrono::steady_clock> latestCookStart =
			  this->m_cookStart.load();
			newCook = latestCookStart != lastCookStart;
			if (newCook) {
				// Reload the time it took
				lastInterCookDuration = std::chrono::duration_cast<std::chrono::microseconds>(
				  latestCookStart - lastCookStart);
				lastCookStart = latestCookStart;

				// We've got a new frame
				packedThisFrame = false;
			}
		}

		// lastCookStart is now updated if it has changed
		{
			std::chrono::time_point<std::chrono::steady_clock> now =
			  std::chrono::steady_clock::now();
			auto elapsed =
			  std::chrono::duration_cast<std::chrono::microseconds>(now - lastCookStart);
			auto left = lastInterCookDuration - elapsed;

			if ((solver.transformedData != nullptr) && (!packedThisFrame) &&
				(extractPosition || extractVelocity || extractError)) {
				const bool shouldPack =
				  newCook ||
				  (left <
				   (lastInterCookDuration /
					2)); /* TODO: use calculation derived from the time it takes to step*/
				;

				// Output
				if (shouldPack) {
					size_t positions_written = 0;
					size_t error_written = 0;
					size_t velocity_written = 0;

					rtori::ExtractOutRequest extractRequest = {
					  .positions =
						rtori::ArrayOutput<float[3]>{.buffer = nullptr,
													 .buffer_size = 0,
													 .written_size = &positions_written},
					  .velocity =
						rtori::ArrayOutput<float[3]>{.buffer = nullptr,
													 .buffer_size = 0,
													 .written_size = &velocity_written },
					  .error = rtori::ArrayOutput<float>{.buffer = nullptr,
													 .buffer_size = 0,
													 .written_size = &error_written	   }
					};

					uint32_t vertex_count = 0;
					{
						QueryOutput queryOutput{.u32_output = &vertex_count};
						rtori::rtori_fold_query_frame(solver.foldFile,
													  solver.frameIndex,
													  rtori::FoldFrameQuery::VerticesCount,
													  &queryOutput);
					}
					/*std::cout << std::format("Outputting {} vertices", vertex_count)
							  << std::endl;*/

					size_t sizeNeededTotal = ((extractPosition ? 3 * vertex_count : 0) +
											  (extractError ? 3 * vertex_count : 0) +
											  (extractVelocity ? 3 * vertex_count : 0));

					const std::unique_lock<std::mutex> lock(this->m_outputMutex,
															std::try_to_lock);
					if (lock.owns_lock()) {
						if (this->m_output.backingBuffer.size() < sizeNeededTotal) {
							this->m_output.backingBuffer.resize(sizeNeededTotal);
						}

						size_t cursor = 0;

						if (extractPosition) {
							// In floats
							const size_t sizeNeeded = 3 /* x y z */ * vertex_count;

							this->m_output.positions =
							  std::make_tuple(static_cast<uint32_t>(cursor),
											  static_cast<uint32_t>(sizeNeeded));

							using val_t = float[3];
							val_t* buffer = reinterpret_cast<val_t*>(
							  this->m_output.backingBuffer.data() + cursor);

							extractRequest.positions.buffer = buffer;
							extractRequest.positions.buffer_size =
							  vertex_count; /* as its a buffer of arrays of 3 elements already
											 */

							cursor += sizeNeeded;
						}

						if (extractVelocity) {
							// In floats
							const size_t sizeNeeded = 3 /* x y z */ * vertex_count;

							this->m_output.velocity =
							  std::make_tuple(static_cast<uint32_t>(cursor),
											  static_cast<uint32_t>(sizeNeeded));

							using val_t = float[3];
							val_t* buffer = reinterpret_cast<val_t*>(
							  this->m_output.backingBuffer.data() + cursor);

							extractRequest.velocity.buffer = buffer;
							extractRequest.velocity.buffer_size =
							  vertex_count; /* as its a buffer of arrays of 3 elements already
											 */

							cursor += sizeNeeded;
						}

						if (extractError) {
							// In floats
							const size_t sizeNeeded = vertex_count;

							this->m_output.velocity =
							  std::make_tuple(static_cast<uint32_t>(cursor),
											  static_cast<uint32_t>(sizeNeeded));

							using val_t = float;
							val_t* buffer = reinterpret_cast<val_t*>(
							  this->m_output.backingBuffer.data() + cursor);

							extractRequest.error.buffer = buffer;
							extractRequest.error.buffer_size = vertex_count;

							cursor += sizeNeeded;
						}

						const rtori::SolverOperationResult result =
						  rtori::rtori_extract(solver.solver, &extractRequest);
						assert((void("extraction should never fail"),
								result == rtori::SolverOperationResult::Success));

						if (this->m_output.positions.has_value()) {
							// Add in the verticesUnchanging from the fold file as we only got
							// the offset
							if (!verticesCachedUnchanging) {
								verticesUnchanging.resize(static_cast<size_t>(vertex_count) *
														  3);

								size_t writtenSize = 0;
								using val_t = float[3];

								rtori::QueryOutput queryOutput = QueryOutput{
								  .vec3f_array_output = {.buffer = reinterpret_cast<val_t*>(
verticesUnchanging.data()),
														 .buffer_size = vertex_count,
														 .written_size = &writtenSize,
														 .offset = 0}
								 };

								rtori::FoldOperationStatus queryStatus =
								  rtori::rtori_fold_query_frame(solver.foldFile,
																solver.frameIndex,
																FoldFrameQuery::VerticesCoords,
																&queryOutput);

								assert(queryStatus == rtori::FoldOperationStatus::Success);
								verticesCachedUnchanging = true;
							}

							auto dest = this->m_output.backingBuffer.data() +
										std::get<0>(this->m_output.positions.value());

							for (size_t i = 0; i < vertex_count * 3; i++) {
								dest[i] += verticesUnchanging[i];
							}
						}

						{
							// Get the number of faces
							uint32_t faceCount = 0;

							{
								rtori::QueryOutput faceCountQueryOutput = {.u32_output =
																			 &faceCount};

								rtori::FoldOperationStatus faceCountOperationStatus =
								  rtori::rtori_fold_transformed_query(
									solver.transformedData,
									rtori::TransformedQuery::FacesCount,
									&faceCountQueryOutput);

								assert(faceCountOperationStatus ==
									   rtori::FoldOperationStatus::Success);
							}

							// Ensure we can host that amount of faces
							size_t indicesCount = static_cast<size_t>(faceCount) * 3;
							if (this->m_output.indices.size() != indicesCount) {
								this->m_output.indices.resize(indicesCount);
							}

							using val_t = uint32_t[3];

							size_t faceOutputCount = 0;
							rtori::QueryOutput output = {
							  .vec3u_array_output = {.buffer = reinterpret_cast<val_t*>(
this->m_output.indices.data()),
													 .buffer_size = faceCount,
													 .written_size = &faceOutputCount}
							  };

							rtori::FoldOperationStatus result =
							  rtori::rtori_fold_transformed_query(
								solver.transformedData,
								rtori::TransformedQuery::FacesVertexIndices,
								&output);

							assert(result == rtori::FoldOperationStatus::Success);
							assert(faceOutputCount == faceCount);
						}

						packedThisFrame = true;
					}
				}
			}
		}

		// Input
		if (newCook) {
			const std::unique_lock<std::mutex> lock(this->m_inputMutex, std::try_to_lock);
			if (lock.owns_lock()) {
				const Input& input = this->m_input;
				if (input.inputNumber != lastInputNumber) {
					// TODO: Create a "geometryChanged" method
					if (input.foldFileSource.changed || input.frameIndex.changed) {
						// Invalidate the verticesUnchanging cache
						verticesCachedUnchanging = false;
					}

					// We update our knowledge of what to extract
					// We don't care about the dirty flag as they don't impact anything else
					extractPosition = input.extractPosition.value;
					extractVelocity = input.extractVelocity.value;
					extractError = input.extractError.value;

					if (input.resetFlag) {
						std::cout << "Resetting..." << std::endl;
					}

					SolverImportResult result =
					  solver.update((input.resetFlag || input.foldFileSource.changed)
									  ? std::optional(input.foldFileSource.value)
									  : std::nullopt,
									(input.resetFlag || input.frameIndex.changed)
									  ? std::optional(input.frameIndex.value)
									  : std::nullopt,
									(input.resetFlag || input.foldPercentage.changed)
									  ? std::optional(input.foldPercentage.value)
									  : std::nullopt);

					lastInputNumber = input.inputNumber;
					if (result.kind == SolverImportResultKind::Success) {
						loaded = true;
					} else {
						loaded = false;
						// TODO: report error
						/*std::cout << std::format("Error importing: {}", result.format())
								  << std::endl;*/
					}
				}
			}
		}

		// TODO: ShouldCook logic here

		// Stepping
		if (loaded) {
			std::chrono::time_point<std::chrono::steady_clock> const beforeStep =
			  std::chrono::steady_clock::now();
			auto const elapsed =
			  std::chrono::duration_cast<std::chrono::microseconds>(beforeStep - lastCookStart);
			std::chrono::microseconds left = lastInterCookDuration - elapsed;
			if (left < std::chrono::microseconds(0)) {
				left = std::chrono::microseconds(0);
			}

			float const ratio = 0.66;
			/*std::cout << "Stepping based on previous calculated step duration of "
					  << stepDuration << " and time left " << left << std::endl;*/
			uint32_t stepCount =
			  (stepDuration > std::chrono::microseconds(0))
				? std::clamp(
					static_cast<uint32_t>(
					  std::chrono::duration_cast<std::chrono::microseconds>(
						(ratio *
						 std::chrono::duration_cast<std::chrono::duration<float, std::micro>>(
						   left))) /
					  stepDuration),
					static_cast<uint32_t>(1),
					static_cast<uint32_t>(100))
				: static_cast<uint32_t>(100);

			rtori::SolverOperationResult const stepResult =
			  rtori::rtori_solver_step(solver.solver, stepCount);

			std::chrono::time_point<std::chrono::steady_clock> const afterStep =
			  std::chrono::steady_clock::now();

			if (stepResult != rtori::SolverOperationResult::Success) {
				// ERROR
				std::cout << "ERROR: Solver step failed: "
						  << format_SolverOperationResult(stepResult) << std::endl;
			}

			auto const totalStepDuration = afterStep - beforeStep;
			stepDuration = std::chrono::duration_cast<std::chrono::microseconds>(
			  totalStepDuration / stepCount);
			/*std::cout << "Stepped " << stepCount << " times, taking " << totalStepDuration
					  << "( " << stepDuration << " per step )" << std::endl;*/
		}

		/*using namespace std::chrono_literals;
		std::this_thread::sleep_for(10ms);*/
	}
}
