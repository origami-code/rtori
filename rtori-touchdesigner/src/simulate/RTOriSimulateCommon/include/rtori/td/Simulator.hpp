#ifndef RTORI_TD_SIMULATOR_HPP_
#define RTORI_TD_SIMULATOR_HPP_

#ifdef _MSC_VER
#pragma warning(push, 0)
#endif
#include "CPlusPlus_Common.h"
#ifdef _MSC_VER
#pragma warning(pop)
#endif

#include <optional>
#include <string_view>

#include "rtori/td/SimulationThread.hpp"
#include "rtori_core.hpp"

#include "Interests.hpp"

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

class RTORI_TD_EXPORT Simulator {
  public:
	Simulator(rtori::Context const* context);
	~Simulator();

	/// Polls the simulator for the cook results
	void execute(const TD::OP_Inputs* inputs, const Interests& interests);

	/**
	 * @brief Notify that a pulse parameter has been triggered
	 *
	 * @param name of the parameter, as given to `pulsePressed` of the operator
	 * @return true if the parameter was for the simulator
	 * @return false otherwise
	 */
	bool pulsePressed(const char* name);

	/// Retrieves the output of the simulation
	rtori::rtori_td::OutputGuard query(void);

	/// Sets up the parameter of the OP
	static void setupParameters(
	  TD::OP_ParameterManager* manager,
	  const char* pageName = nullptr);

	int32_t getNumInfoCHOPChans(void);
	void getInfoCHOPChan(int32_t index, TD::OP_InfoCHOPChan* chan);

	bool getInfoDATSize(TD::OP_InfoDATSize* infoSize);
	void getInfoDATEntries(int32_t index, int32_t nEntries, TD::OP_InfoDATEntries* entries);

	void getErrorString(TD::OP_String* error);
	void getInfoPopupString(TD::OP_String* info);

	rtori::Context const* rtoriCtx;

  private:
	rtori::rtori_td::Input consolidateParameters(const TD::OP_Inputs* inputs,
												 const Interests& interests) const;

	rtori::rtori_td::SimulationThread m_simulation;
	Interests m_interests;
};

} // namespace rtori::rtori_td
#endif // !RTORI_TD_SIMULATOR_HPP_
