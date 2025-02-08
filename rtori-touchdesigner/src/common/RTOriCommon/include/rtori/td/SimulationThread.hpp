#pragma once

#include <chrono>
#include <thread>
#include <mutex>

#ifndef __cpp_lib_jthread
#include <atomic>
#endif

#include "Input.hpp"
#include "Output.hpp"

#include <rtori_core.hpp>

#ifdef RTORI_TD_BUILD_SHARED
#ifdef _MSC_VER
#define RTORI_TD_EXPORT __declspec(dllexport)
#else
#define RTORI_TD_EXPORT __attribute__((visibility("default")))
#endif
#else
#ifdef _MSC_VER
#define RTORI_TD_EXPORT __declspec(dllimport)
#else
#define RTORI_TD_EXPORT
#endif
#endif

namespace rtori::rtori_td {

struct OutputGuard {
  public:
	Output const& output;
	OutputGuard(Output const& output, std::unique_lock<std::mutex>&& guard);

  private:
	std::unique_lock<std::mutex> m_guard;
};

class RTORI_TD_EXPORT SimulationThread final {
  public:
	SimulationThread(rtori::Context const* ctx);
	~SimulationThread();

	/// This requests a stop of the worker
	/// This will be done by the destructor anyway
	void requestStopWorker();

	Input const& getInput() const;
	void update(Input);

	OutputGuard getOutput();

	void notifyCook();
	bool isStopRequested();
  private:
	/// This should be called from the newly created thread
	void runWorker();

#ifdef __cpp_lib_jthread
	std::jthread m_threadHandler;
#else
	std::thread m_threadHandler;
	std::atomic<bool> m_stopRequestFlag;
#endif

	std::condition_variable m_inputCondVar;
	std::mutex m_inputMutex;
	Input m_input;

	std::mutex m_outputMutex;
	Output m_output;

	rtori::Context const* m_ctx;

	/// This is raised on the beginning of every cook, with the exact timestamp
	/// This allows for calibrating the timing of the thread.
	/// Indeed, a change in this is detected and used as a marker for a new cook starting
	/// From it:
	/// - The inter-cook time is calculated
	/// - The time left is derived (less as it's not in perfect sync)
	/// - The number of steps that should be done is computed (from dt)
	std::atomic<std::chrono::time_point<std::chrono::steady_clock>> m_cookStart;
};

}; // namespace rtori::rtori_td
