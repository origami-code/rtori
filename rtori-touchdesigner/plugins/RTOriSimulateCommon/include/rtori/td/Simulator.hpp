#ifndef RTORI_TD_SIMULATOR_HPP_
#define RTORI_TD_SIMULATOR_HPP_

#include "CPlusPlus_Common.h"

#include <optional>
#include <string_view>

#include "rtori/td/SimulationThread.hpp"
#include "rtori_core.hpp"

namespace rtori::rtori_td {

class Simulator {
  public:
	Simulator();
	virtual ~Simulator();

	/// Polls the simulator for the cook results
	void execute(const TD::OP_Inputs* inputs);

	/// Retrieves the output of the simulation
	rtori::rtori_td::OutputGuard query(void);

	/// Sets up the parameter of the OP
	static void setupParameters(
	  TD::OP_ParameterManager* manager,
	  const char* pageName = nullptr);
	
	void getInfoCHOPChan(TD::OP_InfoCHOPChan* chan);
	int32_t getNumInfoCHOPChans(void);

	bool getInfoDATSize(TD::OP_InfoDATSize* infoSize);
	void getInfoDATEntries(int32_t index, int32_t nEntries, TD::OP_InfoDATEntries* entries);

	void getErrorString(TD::OP_String* error);
	void getInfoPopupString(TD::OP_String* info);

	rtori::Context const* rtoriCtx;

  private:
	rtori::rtori_td::Input consolidateParameters(const TD::OP_Inputs* inputs) const;
	rtori::rtori_td::SimulationThread m_simulation;
};

} // namespace rtori::rtori_td
#endif // !RTORI_TD_SIMULATOR_HPP_
