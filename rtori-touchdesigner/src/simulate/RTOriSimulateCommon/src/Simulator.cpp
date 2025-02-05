/* rtori touchdesigner */

#include "rtori/td/Interests.hpp"
#include "rtori/td/SimulateOP.hpp"
#include "rtori/td/Simulator.hpp"
#include "rtori/td/InfoCHOP.hpp"

#include <cassert>

#include "CPlusPlus_Common.h"

#include "rtori/td/Context.hpp"
#include "rtori_core.hpp"

#include <cassert>

using namespace TD;
using namespace rtori::rtori_td;

constexpr float DEFAULT_IDLE_THRESHOLD = 0.0001f;

constexpr const char* PARAMETER_KEY_SOURCE_SIMULATION = "Sourcesimulation";
constexpr const char* PARAMETER_KEY_FOLD_SOURCE = "Foldsource";
constexpr const char* PARAMETER_KEY_FOLD_FRAME_INDEX = "Foldframeindex";
constexpr const char* PARAMETER_KEY_FOLD_PERCENTAGE = "Foldpercentage";
constexpr const char* PARAMETER_KEY_IDLE_THRESHOLD = "Idlethreshold";

constexpr const char* PARAMETER_KEYS_SIMULATION[] = {PARAMETER_KEY_FOLD_SOURCE,
													 PARAMETER_KEY_FOLD_FRAME_INDEX,
													 PARAMETER_KEY_FOLD_FRAME_INDEX,
													 PARAMETER_KEY_FOLD_PERCENTAGE,
													 PARAMETER_KEY_IDLE_THRESHOLD};

Simulator::Simulator(rtori::Context const* ctx)
	: m_simulation(rtori::rtori_td::SimulationThread(ctx)), rtoriCtx(ctx) {}

Simulator::~Simulator() {}

void Simulator::execute(const TD::OP_Inputs* inputs, const Interests& interests) {
	// TODO: Always take account of interests

	// It's time to cook ! Let's mark it
	this->m_simulation.notifyCook();

	// If Sourcesimulation is active, we disable the other parameters
	if (inputs->getParString(PARAMETER_KEY_SOURCE_SIMULATION) != nullptr) {
		for (size_t i = 0; i < sizeof(PARAMETER_KEYS_SIMULATION) / sizeof(const char*); i++) {
			inputs->enablePar(PARAMETER_KEYS_SIMULATION[i], false);
		}
	} else {
		// We convert the parameters into an Input
		const rtori::rtori_td::Input consolidated = consolidateParameters(inputs, interests);
		if (consolidated.changed()) {
			this->m_simulation.update(consolidated);
		}
	}
}

rtori::rtori_td::OutputGuard Simulator::query(void) {
	return this->m_simulation.getOutput();
}

constexpr char const* PARAMETERS_PAGE_NAME = "Simulation Settings";

void Simulator::setupParameters(TD::OP_ParameterManager* manager, const char* page) {
	if (page == nullptr) {
		page = PARAMETERS_PAGE_NAME;
	}

	{
		OP_StringParameter parameter;

		parameter.name = PARAMETER_KEY_SOURCE_SIMULATION;
		parameter.page = page;
		parameter.label = "Source Simulation";

		const OP_ParAppendResult res = manager->appendOP(parameter);
		assert(res == OP_ParAppendResult::Success);
	}

	{
		OP_NumericParameter parameter;

		parameter.name = "Simulationmode";
		parameter.page = page;
		parameter.label = "Simulation Mode";

		const OP_ParAppendResult res = manager->appendToggle(parameter);
		assert(res == OP_ParAppendResult::Success);
	}

	{
		OP_StringParameter parameter;
		parameter.name = PARAMETER_KEY_FOLD_SOURCE;
		parameter.page = page;
		parameter.label = "Fold Source";

		const OP_ParAppendResult res = manager->appendString(parameter);
		assert(res == OP_ParAppendResult::Success);
	}

	{
		OP_NumericParameter parameter;

		parameter.name = PARAMETER_KEY_FOLD_FRAME_INDEX;
		parameter.page = page;
		parameter.label = "Fold Frame Index";

		const OP_ParAppendResult res = manager->appendToggle(parameter);
		assert(res == OP_ParAppendResult::Success);
	}

	{
		OP_NumericParameter parameter;
		parameter.name = PARAMETER_KEY_FOLD_PERCENTAGE;
		parameter.page = page;
		parameter.label = "Crease Percentage";

		parameter.clampMins[0] = true;
		parameter.clampMaxes[0] = true;

		parameter.minValues[0] = -1.0f;
		parameter.maxValues[0] = 1.0;

		parameter.minSliders[0] = -1.0f;
		parameter.maxSliders[0] = 1.0f;
		parameter.defaultValues[0] = 0.0f;

		const OP_ParAppendResult res = manager->appendFloat(parameter, 1);
		assert(res == OP_ParAppendResult::Success);
	}

	{
		auto parameter = OP_NumericParameter();
		parameter.name = PARAMETER_KEY_IDLE_THRESHOLD;
		parameter.page = page;
		parameter.label = "Idle threshold";

		parameter.clampMins[0] = true;
		parameter.minValues[0] = 0.0;
		parameter.minSliders[0] = 0.0;
		parameter.defaultValues[0] = DEFAULT_IDLE_THRESHOLD;

		const OP_ParAppendResult res = manager->appendFloat(parameter, 1);
		assert(res == OP_ParAppendResult::Success);
	}
}

#include "rtori/td/InfoCHOP.hpp"

constexpr int32_t INFO_CHOP_CHANNEL_COUNT =
  sizeof(INFO_CHOP_CHANNEL_NAMES) / sizeof(const char*);

int32_t Simulator::getNumInfoCHOPChans() {
	// Return the channels
	// In particular, total node error, dt, that kind of things
	return INFO_CHOP_CHANNEL_COUNT;
}

void Simulator::getInfoCHOPChan(int32_t index, TD::OP_InfoCHOPChan* chan) {
	assert(index < INFO_CHOP_CHANNEL_COUNT);

	chan->name->setString(INFO_CHOP_CHANNEL_NAMES[index]);
	chan->value = 0.0f;
}

bool Simulator::getInfoDATSize(TD::OP_InfoDATSize* infoSize) {
	// TODO: Return info
	return false;
}

void Simulator::getInfoDATEntries(int32_t index,
								  int32_t nEntries,
								  TD::OP_InfoDATEntries* entries) {
	// Fill in info (in particular, if point-per-node is selected, output the UVs
	// per prims)
}

void Simulator::getErrorString(TD::OP_String* error) {
	// TODO
}

void Simulator::getInfoPopupString(TD::OP_String* info) {
	info->setString("Not loaded");
}

rtori::rtori_td::Input Simulator::consolidateParameters(const TD::OP_Inputs* inputs,
														const Interests& interests) const {
	const rtori::rtori_td::Input& cachedInput = this->m_simulation.getInput();

	rtori::rtori_td::Input input = {
	  .inputNumber = cachedInput.inputNumber,
	  .foldFileSource = cachedInput.foldFileSource.update(
		std::string(inputs->getParString(PARAMETER_KEY_FOLD_SOURCE))),
	  .frameIndex =
		cachedInput.frameIndex.update(inputs->getParInt(PARAMETER_KEY_FOLD_FRAME_INDEX)),
	  .foldPercentage = cachedInput.foldPercentage.update(
		static_cast<float>(inputs->getParDouble(PARAMETER_KEY_FOLD_PERCENTAGE))),
	  .extractPosition = cachedInput.extractPosition.update(interests.position),
	  .extractError = cachedInput.extractError.update(interests.error),
	  .extractVelocity = cachedInput.extractVelocity.update(interests.velocity)};

	if (input.changed()) {
		input.inputNumber += 1;
	}

	return input;
}
