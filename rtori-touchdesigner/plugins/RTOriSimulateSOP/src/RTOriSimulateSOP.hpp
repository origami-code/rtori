#ifndef __SimulateSOP__
#define __SimulateSOP__

#include "SOP_CPlusPlusBase.h"
#include <atomic>
#include <mutex>
#include <thread>
#include <vector>

#include "Input.hpp"
#include "Output.hpp"

#include "rtori_core.hpp"

using namespace rtori_td;

/// This SOP is a generator and it takes no input, though it does take a lot of parameters
class SimulateSOP : public TD::SOP_CPlusPlusBase {
  public:
	SimulateSOP(const TD::OP_NodeInfo* info);
	virtual ~SimulateSOP();

	virtual void getGeneralInfo(TD::SOP_GeneralInfo*, const TD::OP_Inputs*, void*) override;

	virtual void execute(TD::SOP_Output*, const TD::OP_Inputs*, void*) override;

	virtual void executeVBO(TD::SOP_VBOOutput*, const TD::OP_Inputs*, void*) override;

	virtual void setupParameters(TD::OP_ParameterManager*, void*) override;

	virtual int32_t getNumInfoCHOPChans(void* reserved1) override;

	virtual void getInfoCHOPChan(int32_t index,
								 TD::OP_InfoCHOPChan* chan,
								 void* reserved1) override;

	virtual bool getInfoDATSize(TD::OP_InfoDATSize* infoSize, void* reserved1) override;

	virtual void getInfoDATEntries(int32_t index,
								   int32_t nEntries,
								   TD::OP_InfoDATEntries* entries,
								   void* reserved1) override;

	virtual void getErrorString(TD::OP_String* error, void* reserved1) override;

	virtual void getInfoPopupString(TD::OP_String* info, void* reserved1) override;

  private:
	Input consolidateParameters(const TD::OP_Inputs* inputs) const;

	/// Runs the core of the worker
	void runWorkerThread();
	/// Returns true if we should pack data out for output
	bool shouldPack() const;

	/// The thread _actually_ running the simulation
	std::thread m_simulationThread;

	/// This mutex protects the input & output data from being accessed concurrently
	/// between the TD thread and the worker thread
	std::atomic<int32_t> m_cookRequest;
	std::atomic_flag m_workerShouldExit;

	std::mutex m_inputMutex;
	Input m_input;

	std::mutex m_outputMutex;
	Output m_output;

	/// No need to lock the mutex to get that one
	/// We can remove it as we only change it from the same thread that
	/// also needs the cachedInput
	Input m_cachedInput;
};

#endif // !__SimulateSOP__
