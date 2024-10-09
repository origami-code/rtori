/* rtori touchdesigner */

#include <chrono>
#include <cstddef>
#include <iterator>
#include <mutex>
#include <thread>
#include <cassert>
#include <iostream>

#ifndef RTORI_TOUCHDESIGNER_SIMULATE_SOP_VERSION_MAJOR
#warning "RTORI_TOUCHDESIGNER_SIMULATE_SOP_VERSION_MAJOR undefined, setting to 0"
#define RTORI_TOUCHDESIGNER_SIMULATE_SOP_VERSION_MAJOR 0
#endif

#ifndef RTORI_TOUCHDESIGNER_SIMULATE_SOP_VERSION_MINOR
#warning "RTORI_TOUCHDESIGNER_SIMULATE_SOP_VERSION_MINOR undefined, setting to 0"
#define RTORI_TOUCHDESIGNER_SIMULATE_SOP_VERSION_MINOR 0
#endif

#include "RTOriSimulateSOP.hpp"
#include "CPlusPlus_Common.h"

#include "rtori_core.hpp"

#ifdef _MSC_VER
#define MSVC
#endif

#include <optional>
#include <cassert>
#include <format>

using namespace TD;

// These functions are basic C function, which the DLL loader can find
// much easier than finding a C++ Class.
// The DLLEXPORT prefix is needed so the compile exports these functions from
// the .dll you are creating
extern "C" {

DLLEXPORT
void FillSOPPluginInfo(SOP_PluginInfo* info) {
	// For more information on SOP_PluginInfo see SOP_CPlusPlusBase.h

	// Always set this to SOPCPlusPlusAPIVersion.
	info->apiVersion = SOPCPlusPlusAPIVersion;

	// For more information on OP_CustomOPInfo see CPlusPlus_Common.h
	OP_CustomOPInfo& customInfo = info->customOPInfo;

	// Unique name of the node which starts with an upper case letter, followed by
	// lower case letters or numbers
	customInfo.opType->setString("Rtorisimulate");
	// English readable name
	customInfo.opLabel->setString("RTOri Simulate");
	// Will be turned into a 3 letter icon on the nodes
	customInfo.opIcon->setString("ROS");
	customInfo.majorVersion = RTORI_TOUCHDESIGNER_SIMULATE_SOP_VERSION_MAJOR;
	customInfo.minorVersion = RTORI_TOUCHDESIGNER_SIMULATE_SOP_VERSION_MINOR;

	// Information of the author of the node
	customInfo.authorName->setString("Ars Electronica Futurelab");
	customInfo.authorEmail->setString("futurelab@ars.electronica.art");

	// This SOP takes inputs by parameter (it is a generator)
	customInfo.minInputs = 0;
	customInfo.maxInputs = 0;
}

static int32_t instance_count = 0;
static rtori::Context const* rtori_ctx = nullptr;

#include <stdlib.h>

static void* rtori_alloc(const void* alloc_ctx, size_t size, size_t alignment) {
#ifdef MSVC
	return _aligned_malloc(size, alignment);
#else
	return aligned_malloc(size, alignment);
#endif
}

static void rtori_dealloc(const void* dealloc_ctx, void* ptr, size_t size, size_t alignment) {
#ifdef MSVC
	_aligned_free(ptr);
#else
	aligned_free(ptr, size, alignment);
#endif
}

DLLEXPORT
SOP_CPlusPlusBase* CreateSOPInstance(const OP_NodeInfo* info) {
	if (instance_count == 0) {
		rtori_ctx = rtori::rtori_ctx_init(nullptr);
	}

	instance_count += 1;

	// Return a new instance of your class every time this is called.
	// It will be called once per SOP that is using the .dll
	return new SimulateSOP(info);
}

DLLEXPORT
void DestroySOPInstance(SOP_CPlusPlusBase* instance) {
	// Delete the instance here, this will be called when
	// Touch is shutting down, when the SOP using that instance is deleted, or
	// if the SOP loads a different DLL
	delete (SimulateSOP*)instance;

	assert(instance_count >= 1);
	instance_count -= 1;

	if (instance_count == 0) {
		assert(rtori_ctx != nullptr);
		rtori::rtori_ctx_deinit(rtori_ctx);
	}
};
}

constexpr float DEFAULT_IDLE_THRESHOLD = 0.0001f;
constexpr const char* PARAMETER_KEY_FOLD_SOURCE = "Foldsource";
constexpr const char* PARAMETER_KEY_POSITION = "Extractposition";
constexpr const char* PARAMETER_KEY_ERROR = "Extracterror";
constexpr const char* PARAMETER_KEY_VELOCITY = "Extractvelocity";

SimulateSOP::SimulateSOP(const OP_NodeInfo* _info) {
	this->m_simulationThread = std::thread(&SimulateSOP::runWorkerThread, this);
};

SimulateSOP::~SimulateSOP() {
	this->m_workerShouldExit.test_and_set();

	// joining thread - may block
	if (this->m_simulationThread.joinable()) {
		this->m_simulationThread.join();
	}
};

void SimulateSOP::getGeneralInfo(SOP_GeneralInfo* ginfo, const TD::OP_Inputs* inputs, void*) {
	// This will cause the node to cook every frame if the output is used
	ginfo->cookEveryFrameIfAsked = true;

	// Direct shape to GPU loading if asked
	ginfo->directToGPU = inputs->getParInt("Gpudirect") != 0 ? true : false;

	// New plugins should always use CCW winding.
	ginfo->winding = SOP_Winding::CCW;
}

void SimulateSOP::execute(SOP_Output* output, const TD::OP_Inputs* inputs, void*) {
	// It's time to cook ! Let's mark it
	this->m_cookRequest++;

	// We convert the parameters into an Input
	const Input consolidated = consolidateParameters(inputs);
	if (consolidated.changed()) {
		std::cout << "Changed ! Input Number " << consolidated.inputNumber << std::endl;

		const std::unique_lock<std::mutex> lock(this->m_inputMutex, std::try_to_lock);
		if (lock.owns_lock()) {
			this->m_input = consolidated;
			this->m_cachedInput = consolidated;
		}
	}

	// We read out the output
	Output simulationOutput;
	{
		const std::lock_guard<std::mutex> lock(this->m_inputMutex);
		simulationOutput = Output(this->m_output);
	}

	// TODO:
	// Refactor into a single DLL called by both SimulateSOP and SimulateDAT
	// Computation should happen on another thread (launched at the start of
	// SimulateSOP, and killed when torn down)
	// There is a target simulation speed (see oslib-streamer)
	// A flag indicates that it is ready to harvest the data
	// If it is, then the data is copied out here
	// If the flag is skipped, then we're not fast enough to be realtime
	// If the mode is "dynamic speed (realtime rendering)", then we mark that as a
	// warning, and adjust the speed down. Otherwise if it's "static speed (might
	// skip frames)" we don't care. In both cases the speed parameter is followed
	// as a target taking into account the frame time (given as a parameter,
	// coming from global) and the dt
	// That also means we need to keep a cache
	// The attributes like UVs are kept the same (in point-per-vertex mode)

	const char* normalsCustomAttributeName = "N";
	const char* errorCustomAttributeName = "Error";

	// Set positions

	// Setting the normals (they are an attribute of nodes, which are points here)
	if (simulationOutput.positions.has_value()) {
		SOP_CustomAttribData nodeNormalAttrib{normalsCustomAttributeName, 3, AttribType::Float};
		nodeNormalAttrib.floatData = simulationOutput.backingBuffer.data() +
									 std::get<0>(simulationOutput.positions.value());
		output->setCustomAttribute(&nodeNormalAttrib, output->getNumPoints());
	}

	// Setting the error (they are an attribute of nodes as well)
	if (simulationOutput.error.has_value()) {
		SOP_CustomAttribData nodeErrorAttrib{"Error", 1, AttribType::Float};
		nodeErrorAttrib.floatData =
		  simulationOutput.backingBuffer.data() + std::get<0>(simulationOutput.error.value());
		output->setCustomAttribute(&nodeErrorAttrib, output->getNumPoints());
	}

	// Unfortunately, UVs need to be per-vertex, and I haven't found a way to set
	// vertex attributes from an SOP
	// See
	// https://forum.derivative.ca/t/c-trouble-adding-more-than-1-set-of-uv-coords-using-settexcoord/257147
	// So we need to duplicate points for each triangle if we want to set UVs
	// That could be a split mode of that SOP
	// OR we call python which can do it
}

void SimulateSOP::executeVBO(SOP_VBOOutput* output, const TD::OP_Inputs* inputs, void*) {
	// ShapeMenuItems shape = myParms.evalShape(inputs);
	// Color color = myParms.evalColor(inputs);

	output->enableNormal();
	output->enableTexCoord(1);

	// output->allocVBO(numVertices, int32_t numIndices, VBOBufferMode.Dynamic);

	/*
	switch (shape) {
	case ShapeMenuItems::Point: {
	  myShapeGenerator.outputDotVBO(output);
	  break;
	}
	case ShapeMenuItems::Line: {
	  myShapeGenerator.outputLineVBO(output);
	  break;
	}
	case ShapeMenuItems::Square: {
	  myShapeGenerator.outputSquareVBO(output);
	  break;
	}
	case ShapeMenuItems::Cube:
	default: {
	  myShapeGenerator.outputCubeVBO(output);
	  break;
	}
	}

	int numVertices = myShapeGenerator.getLastVBONumVertices();
  */
	Color* colors = output->getColors();
	/*
	for (int i = 0; i < numVertices; ++i) {
	  colors[i] = color;
	}*/

	output->setBoundingBox(BoundingBox(-1, -1, -1, 1, 1, 1));
	output->updateComplete();
}

void SimulateSOP::setupParameters(TD::OP_ParameterManager* manager, void*) {
	// myParms.setup(manager);
	{
		OP_NumericParameter parameter;

		parameter.name = "Gpudirect";
		parameter.label = "GPU Direct";

		const OP_ParAppendResult res = manager->appendToggle(parameter);
		assert(res == OP_ParAppendResult::Success);
	}

	{
		OP_NumericParameter parameter;

		parameter.name = "Simulationmode";
		parameter.label = "Simulation Mode";

		const OP_ParAppendResult res = manager->appendToggle(parameter);
		assert(res == OP_ParAppendResult::Success);
	}

	{
		OP_StringParameter parameter;
		parameter.name = PARAMETER_KEY_FOLD_SOURCE;
		parameter.label = "Fold Source";

		const OP_ParAppendResult res = manager->appendString(parameter);
		assert(res == OP_ParAppendResult::Success);
	}

	{
		OP_NumericParameter parameter;
		parameter.name = PARAMETER_KEY_POSITION;
		parameter.label = "Extract position";
		parameter.defaultValues[0] = 1;

		const OP_ParAppendResult res = manager->appendToggle(parameter);
		assert(res == OP_ParAppendResult::Success);
	}

	{
		OP_NumericParameter parameter;
		parameter.name = PARAMETER_KEY_VELOCITY;
		parameter.label = "Extract velocity";
		parameter.defaultValues[0] = 0;

		const OP_ParAppendResult res = manager->appendToggle(parameter);
		assert(res == OP_ParAppendResult::Success);
	}

	{
		OP_NumericParameter parameter;
		parameter.name = PARAMETER_KEY_ERROR;
		parameter.label = "Extract Error";
		parameter.defaultValues[0] = 0;

		const OP_ParAppendResult res = manager->appendToggle(parameter);
		assert(res == OP_ParAppendResult::Success);
	}

	{
		OP_NumericParameter parameter;
		parameter.name = "Creasepercentage";
		parameter.label = "Crease Percentage";

		parameter.clampMins[0] = true;
		parameter.clampMaxes[0] = true;
		parameter.minSliders[0] = -1.0f;
		parameter.maxSliders[0] = 1.0f;
		parameter.defaultValues[0] = 0.0f;

		const OP_ParAppendResult res = manager->appendFloat(parameter, 1);
		assert(res == OP_ParAppendResult::Success);
	}

	{
		auto parameter = OP_NumericParameter();
		parameter.name = "Idlethreshold";
		parameter.label = "Idle threshold";
		parameter.defaultValues[0] = DEFAULT_IDLE_THRESHOLD;
	}
}

constexpr int32_t INFO_CHOP_CHANNEL_LOADED_INDEX = 0;
constexpr const char* INFO_CHOP_CHANNEL_LOADED_NAME = "rtori_loaded";

constexpr int32_t INFO_CHOP_CHANNEL_DT_INDEX = 1;
constexpr const char* INFO_CHOP_CHANNEL_DT_NAME = "rtori_dt";

constexpr int32_t INFO_CHOP_CHANNEL_STEPS_PER_COOK_INDEX = 2;
constexpr const char* INFO_CHOP_CHANNEL_STEPS_PER_COOK_NAME = "rtori_steps_per_cook";

constexpr int32_t INFO_CHOP_CHANNEL_SIMULATION_TIME_INDEX = 3;
constexpr const char* INFO_CHOP_CHANNEL_SIMULATION_TIME_NAME = "rtori_simulation_time";

constexpr int32_t INFO_CHOP_CHANNEL_TOTAL_STEP_TIME_INDEX = 4;
constexpr const char* INFO_CHOP_CHANNEL_TOTAL_STEP_TIME_NAME = "rtori_total_step_time";

constexpr int32_t INFO_CHOP_CHANNEL_EXTRACT_TIME_INDEX = 5;
constexpr const char* INFO_CHOP_CHANNEL_EXTRACT_TIME_NAME = "rtori_extract_time";

constexpr int32_t INFO_CHOP_CHANNEL_RUNNING_INDEX = 6;
constexpr const char* INFO_CHOP_CHANNEL_RUNNING_NAME = "rtori_running";

constexpr int32_t INFO_CHOP_CHANNEL_MAX_VELOCITY_INDEX = 7;
constexpr const char* INFO_CHOP_CHANNEL_MAX_VELOCITY_NAME = "rtori_max_velocity";

constexpr int32_t INFO_CHOP_CHANNEL_MAX_ERROR_INDEX = 8;
constexpr const char* INFO_CHOP_CHANNEL_MAX_ERROR_NAME = "rtori_max_error";

constexpr const char* INFO_CHOP_CHANNEL_NAMES[]{INFO_CHOP_CHANNEL_LOADED_NAME,
												INFO_CHOP_CHANNEL_DT_NAME,
												INFO_CHOP_CHANNEL_STEPS_PER_COOK_NAME,
												INFO_CHOP_CHANNEL_SIMULATION_TIME_NAME,
												INFO_CHOP_CHANNEL_TOTAL_STEP_TIME_NAME,
												INFO_CHOP_CHANNEL_EXTRACT_TIME_NAME,
												INFO_CHOP_CHANNEL_RUNNING_NAME,
												INFO_CHOP_CHANNEL_MAX_VELOCITY_NAME,
												INFO_CHOP_CHANNEL_MAX_ERROR_NAME};

constexpr int32_t INFO_CHOP_CHANNEL_COUNT =
  sizeof(INFO_CHOP_CHANNEL_NAMES) / sizeof(const char*);

int32_t SimulateSOP::getNumInfoCHOPChans(void* reserved1) {
	// Return the channels
	// In particular, total node error, dt, that kind of things
	return INFO_CHOP_CHANNEL_COUNT;
}

void SimulateSOP::getInfoCHOPChan(int32_t index, TD::OP_InfoCHOPChan* chan, void* reserved1) {
	assert(index < INFO_CHOP_CHANNEL_COUNT);

	chan->name->setString(INFO_CHOP_CHANNEL_NAMES[index]);
	chan->value = 0.0f;
}

bool SimulateSOP::getInfoDATSize(TD::OP_InfoDATSize* infoSize, void* reserved1) {
	// TODO: Return info
	return false;
}

void SimulateSOP::getInfoDATEntries(int32_t index,
									int32_t nEntries,
									TD::OP_InfoDATEntries* entries,
									void* reserved1) {
	// Fill in info (in particular, if point-per-node is selected, output the UVs
	// per prims)
}

void SimulateSOP::getErrorString(TD::OP_String* error, void* reserved1) {
	// TODO
}

void SimulateSOP::getInfoPopupString(TD::OP_String* info, void* reserved1) {
	info->setString("Not loaded");
}

Input SimulateSOP::consolidateParameters(const TD::OP_Inputs* inputs) const {
	Input input = {
	  .inputNumber = this->m_cachedInput.inputNumber,
	  .foldFileSource = this->m_cachedInput.foldFileSource.update(
		std::string(inputs->getParString(PARAMETER_KEY_FOLD_SOURCE))),
	  .extractPosition = this->m_cachedInput.extractPosition.update(
		inputs->getParInt(PARAMETER_KEY_POSITION) != 0),
	  .extractError =
		this->m_cachedInput.extractError.update(inputs->getParInt(PARAMETER_KEY_ERROR) != 0),
	  .extractVelocity = this->m_cachedInput.extractVelocity.update(
		inputs->getParInt(PARAMETER_KEY_VELOCITY) != 0),
	};

	if (input.changed()) {
		input.inputNumber += 1;
	}

	return input;
}

enum class ImportInputIntoSolverResultKind {
	Success,
	FoldEmpty,
	FoldParseError,
	FoldLoadError,
};

struct ImportInputIntoSolverResult {
  public:
	ImportInputIntoSolverResultKind kind;
	union {
		rtori::JsonParseError parseError;
	} payload;

	std::string format() const {
		switch (kind) {
		case ImportInputIntoSolverResultKind::FoldParseError: {
			rtori::JsonParseError const& details = this->payload.parseError;
			return std::format("[ERROR] Fold parse error \"{}\" on line {}, column {}",
							   (int32_t)details.category,
							   details.line,
							   details.column);
		}
		case ImportInputIntoSolverResultKind::FoldLoadError:
			return std::string("[ERROR] Fold load error");
		case ImportInputIntoSolverResultKind::FoldEmpty:
			return std::string("[ERROR] Fold input is empty");
		case ImportInputIntoSolverResultKind::Success:
			return std::string("[SUCCESS] Fold loaded successfully");
		default:
			return std::string("[ERROR] Unknown error kind");
		}
	}
};

/// [TD-THREAD]
ImportInputIntoSolverResult importInputIntoSolver(rtori::Solver const* solver,
												  const Input& input) {
	const std::string_view fold = input.foldFileSource.value;

	if (fold.length() == 0) {
		return ImportInputIntoSolverResult{.kind = ImportInputIntoSolverResultKind::FoldEmpty};
	}

	// First, parse
	const rtori::FoldParseResult foldParseResult =
	  rtori::rtori_fold_parse(rtori_ctx,
							  reinterpret_cast<const uint8_t*>(fold.data()),
							  fold.length());

	if (foldParseResult.status != rtori::FoldParseStatus::Success) {
		std::cout << "Error while parsing fold file" << std::endl;

		if (foldParseResult.status == rtori::FoldParseStatus::Error) {
			return ImportInputIntoSolverResult{
			  .kind = ImportInputIntoSolverResultKind::FoldParseError,
			  .payload{.parseError = foldParseResult.payload.error}};
		} else if (foldParseResult.status == rtori::FoldParseStatus::Empty) {
			return ImportInputIntoSolverResult{
			  .kind = ImportInputIntoSolverResultKind::FoldEmpty,
			};
		} else {
			assert(false);
		}
	}
	std::cout << "Parsed fold file" << std::endl;

	const rtori::FoldFile* foldFile = foldParseResult.payload.file;

	// Then load
	const rtori::SolverOperationResult solverLoadResult =
	  rtori::rtori_solver_load_from_fold(solver, foldFile, 0);
	if (solverLoadResult != rtori::SolverOperationResult::Success) {
		std::cout << "Error while loading fold file" << std::endl;

		return ImportInputIntoSolverResult{.kind =
											 ImportInputIntoSolverResultKind::FoldLoadError};
	}
	std::cout << "Loaded fold file" << std::endl;

	// Done
	return ImportInputIntoSolverResult{.kind = ImportInputIntoSolverResultKind::Success};
}

bool SimulateSOP::shouldPack() const {
	return true;
}

void SimulateSOP::runWorkerThread() {
	int64_t outputCounter = 0;
	int64_t lastInputNumber = 0;

	bool extractPosition = false;
	bool extractVelocity = false;
	bool extractError = false;

	const rtori::Parameters solverCreationParams = {.solver =
													  rtori::SolverKind::OrigamiSimulator,
													.backend = rtori::BackendFlags_ANY};

	rtori::Solver const* solver =
	  rtori::rtori_ctx_create_solver(rtori_ctx, &solverCreationParams);
	bool hasLoaded = false;

	while (!this->m_workerShouldExit.test()) {
		{
			const std::unique_lock<std::mutex> lock(this->m_inputMutex, std::try_to_lock);
			if (lock.owns_lock()) {
				const Input& input = this->m_input;
				if (input.inputNumber != lastInputNumber) {
					extractPosition = input.extractPosition.value;
					extractVelocity = input.extractVelocity.value;
					extractError = input.extractError.value;

					ImportInputIntoSolverResult result = importInputIntoSolver(solver, input);

					if (result.kind == ImportInputIntoSolverResultKind::Success) {
						hasLoaded = true;
					} else {
						// TODO: report error
						std::cout << std::format("Error importing: {}", result.format())
								  << std::endl;
					}
				}
			}
		}

		rtori::SolverOperationResult stepResult = rtori::rtori_solver_step(solver, 1);
		if (stepResult != rtori::SolverOperationResult::Success) {
			// ERROR
			std::cout << "ERROR: Solver step failed" << (uint32_t)stepResult << std::endl;
		}
		if (hasLoaded && (extractPosition || extractVelocity || extractError) &&
			this->shouldPack()) {
			size_t positions_written = 0;
			size_t error_written = 0;
			size_t velocity_written = 0;

			rtori::ExtractOutRequest extractRequest = {
			  .positions = rtori::ArrayOutput<float[3]>{.buffer = nullptr,
														.buffer_size = 0,
														.written_size = &positions_written},
			  .velocity = rtori::ArrayOutput<float[3]>{.buffer = nullptr,
														.buffer_size = 0,
														.written_size = &velocity_written },
			  .error = rtori::ArrayOutput<float>{.buffer = nullptr,
														.buffer_size = 0,
														.written_size = &error_written	   }
			};

			int32_t vertex_count = 0; // TODO: Query number of vertices

			const std::unique_lock<std::mutex> lock(this->m_outputMutex, std::try_to_lock);
			if (lock.owns_lock()) {
				size_t cursor = 0;

				if (extractPosition) {
					// In floats
					const size_t sizeNeeded = 3 /* x y z */ * vertex_count;

					this->m_output.positions =
					  std::make_tuple(static_cast<uint32_t>(cursor),
									  static_cast<uint32_t>(sizeNeeded));

					using val_t = float[3];
					val_t* buffer =
					  reinterpret_cast<val_t*>(this->m_output.backingBuffer.data() + cursor);

					extractRequest.positions.buffer = buffer;
					extractRequest.positions.buffer_size =
					  vertex_count; /* as its a buffer of arrays */

					cursor += sizeNeeded;
				}

				if (extractVelocity) {
					// In floats
					const size_t sizeNeeded = 3 /* x y z */ * vertex_count;

					this->m_output.velocity =
					  std::make_tuple(static_cast<uint32_t>(cursor),
									  static_cast<uint32_t>(sizeNeeded));

					using val_t = float[3];
					val_t* buffer =
					  reinterpret_cast<val_t*>(this->m_output.backingBuffer.data() + cursor);

					extractRequest.velocity.buffer = buffer;
					extractRequest.velocity.buffer_size =
					  vertex_count; /* as its a buffer of arrays */

					cursor += sizeNeeded;
				}

				if (extractError) {
					// In floats
					const size_t sizeNeeded = vertex_count;

					this->m_output.velocity =
					  std::make_tuple(static_cast<uint32_t>(cursor),
									  static_cast<uint32_t>(sizeNeeded));

					using val_t = float;
					val_t* buffer =
					  reinterpret_cast<val_t*>(this->m_output.backingBuffer.data() + cursor);

					extractRequest.error.buffer = buffer;
					extractRequest.error.buffer_size =
					  vertex_count; /* as its a buffer of arrays */

					cursor += sizeNeeded;
				}

				const rtori::SolverOperationResult result =
				  rtori::rtori_extract(solver, &extractRequest);
				assert((void("extraction should never fail"),
						result == rtori::SolverOperationResult::Success));
			}
		}
	}
}