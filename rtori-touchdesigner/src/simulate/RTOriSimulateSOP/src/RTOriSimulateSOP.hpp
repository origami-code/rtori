#ifndef __SimulateSOP__
#define __SimulateSOP__

#include "SOP_CPlusPlusBase.h"

#include "rtori/td/Simulator.hpp"
#include "rtori/td/SimulateOP.hpp"

#include "rtori_core.hpp"

#include <cstdint>

namespace rtori::rtori_td {

/// This SOP is a generator and it takes no input, though it does take a lot of parameters
class SimulateSOP : public TD::SOP_CPlusPlusBase, public rtori::rtori_td::SimulateOP {
  public:
	SimulateSOP(const TD::OP_NodeInfo* info, rtori::Context const* rtoriCtx);
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

	rtori::Context const* rtoriCtx;

	std::shared_ptr<rtori::rtori_td::Simulator> simulator();

  private:
	std::shared_ptr<rtori::rtori_td::Simulator> m_simulator;
};

} // namespace rtori::rtori_td
#endif // !__SimulateSOP__
