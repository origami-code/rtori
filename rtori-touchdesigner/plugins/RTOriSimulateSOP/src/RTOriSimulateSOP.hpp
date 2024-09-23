/* Shared Use License: This file is owned by Derivative Inc. (Derivative)
 * and can only be used, and/or modified for use, in conjunction with
 * Derivative's TouchDesigner software, and only if you are a licensee who has
 * accepted Derivative's TouchDesigner license or assignment agreement
 * (which also govern the use of this file). You may share or redistribute
 * a modified version of this file provided the following conditions are met:
 *
 * 1. The shared file or redistribution must retain the information set out
 * above and this list of conditions.
 * 2. Derivative's name (Derivative Inc.) or its trademarks may not be used
 * to endorse or promote products derived from this file without specific
 * prior written permission from Derivative.
 */

#ifndef __SimulateSOP__
#define __SimulateSOP__

#include "SOP_CPlusPlusBase.h"
#include <atomic>
#include <mutex>
#include <thread>
#include <vector>

#include "Input.hpp"
#include "Output.hpp"

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

	/// The thread _actually_ running the simulation
	std::thread m_simulationThread;

	std::mutex m_mutex;
	Input m_input;
	Output m_output;

	std::atomic_flag m_shouldCook;
	std::condition_variable m_threadSignaller;

	/// A flag for the thread to exit
	std::atomic_flag m_simulationThreadExitFlag;
};

#endif // !__SimulateSOP__
