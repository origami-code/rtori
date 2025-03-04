/* rtori touchdesigner */

#include "rtori/td/Interests.hpp"
#include "rtori/td/SimulateOP.hpp"
#include "rtori/td/Simulator.hpp"
#include "rtori/td/InfoCHOP.hpp"

#include <cassert>
#include <iostream>

#include "CPlusPlus_Common.h"

#include "rtori/td/Context.hpp"
#include "rtori_core.hpp"

#include <cassert>
#include <chrono>

using namespace TD;
using namespace rtori::rtori_td;

constexpr float DEFAULT_IDLE_THRESHOLD = 0.0001f;

constexpr const char* PARAMETER_KEY_SOURCE_SIMULATION = "Sourcesimulation";
constexpr const char* PARAMETER_KEY_FOLD_SOURCE = "Foldsource";
constexpr const char* PARAMETER_KEY_RESET = "Reset";
constexpr const char* PARAMETER_KEY_RUNNING = "Running";

constexpr const char* PARAMETER_KEY_FOLD_FRAME_INDEX = "Foldframeindex";
constexpr const char* PARAMETER_KEY_FOLD_PERCENTAGE = "Foldpercentage";
constexpr const char* PARAMETER_KEY_IDLE_THRESHOLD = "Idlethreshold";

/*
 * The simulation runs like this:
 * - Fixed: The simulation's runs at a user-specified ratio (`TimeScale`) of the real speed,
 * meaning that a second in the real world is a second in the simulation. The simulator itself
 * might be finished earlier, in which case all is good, or later, in which case it accumulates
 * delay, meaning the speed of the simulation doesn't reach the speed of the real world.
 *
 * - Adaptive: The simulation runs like in "Fixed" as long as it can meet the time budget,
 * otherwise the effective simulation speed is lowered. The `AdaptiveFrameBudget` parameter
 * allows one to set how long of a frame it should take.
 */

constexpr const char* PARAMETER_KEY_TIME_SCALE = "Timescale";
constexpr const char* PARAMETER_KEY_ADAPTIVE = "Adaptive";
constexpr const char* PARAMETER_KEY_FRAME_BUDGET = "Framebudget";

/// Simulation parameters that should only be applied to simulation primaries
constexpr const char* PARAMETER_KEYS_SIMULATION[] = {PARAMETER_KEY_FOLD_SOURCE,
													 PARAMETER_KEY_RESET,
													 PARAMETER_KEY_RUNNING,
													 PARAMETER_KEY_FOLD_FRAME_INDEX,
													 PARAMETER_KEY_FOLD_FRAME_INDEX,
													 PARAMETER_KEY_FOLD_PERCENTAGE,
													 PARAMETER_KEY_IDLE_THRESHOLD,
													 PARAMETER_KEY_TIME_SCALE,
													 PARAMETER_KEY_ADAPTIVE,
													 PARAMETER_KEY_FRAME_BUDGET};

Simulator::Simulator(std::shared_ptr<rtori::Context> ctx)
	: rtoriCtx(ctx), m_simulation(rtori::rtori_td::SimulationThread(ctx)) {}

Simulator::~Simulator() {}

void Simulator::execute(const TD::OP_Inputs* inputs, const Interests& interests) {
	// TODO: Always take account of interests

	// It's time to cook ! Let's mark it
	this->m_simulation.notifyCook();

	// If Sourcesimulation is active, we disable the other parameters
	const char* simulationSourceStr = inputs->getParString(PARAMETER_KEY_SOURCE_SIMULATION);
	const bool opIsPrimary =
	  simulationSourceStr == NULL || std::strlen(simulationSourceStr) == 0;
	for (size_t i = 0; i < sizeof(PARAMETER_KEYS_SIMULATION) / sizeof(const char*); i++) {
		inputs->enablePar(PARAMETER_KEYS_SIMULATION[i], opIsPrimary);
	}

	if (!opIsPrimary) {
		// TODO: Recover data from the simulation, return it
		// For now, not supported
		assert(false);
	}

	// We convert the parameters into an Input and update
	const rtori::rtori_td::Input consolidated = consolidateParameters(inputs, interests);
	if (consolidated.changed()) {
		this->m_simulation.update(consolidated);
	}
}

bool Simulator::pulsePressed(const char* name) {
	if (std::strcmp(name, PARAMETER_KEY_RESET) != 0) {
		std::cout << "Not RESET !" << std::endl;
		return false;
	}

	const rtori::rtori_td::Input& cachedInput = this->m_simulation.getInput();
	auto newInput = rtori::rtori_td::Input(cachedInput);
	newInput.resetFlag = true;
	newInput.inputNumber += 1;
	this->m_simulation.update(newInput);
	std::cout << "Reset sent for update !" << std::endl;

	return true;
}

rtori::rtori_td::OutputGuard Simulator::query(void) {
	return this->m_simulation.getOutput();
}

constexpr char const* PARAMETERS_PAGE_NAME = "Simulation";

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
		OP_StringParameter parameter;
		parameter.name = PARAMETER_KEY_FOLD_SOURCE;
		parameter.page = page;
		parameter.label = "Fold Source";

		const OP_ParAppendResult res = manager->appendString(parameter);
		assert(res == OP_ParAppendResult::Success);
	}

	{
		auto parameter = OP_NumericParameter();
		parameter.name = PARAMETER_KEY_RESET;
		parameter.page = page;
		parameter.label = "Reset";

		const OP_ParAppendResult res = manager->appendPulse(parameter);
		assert(res == OP_ParAppendResult::Success);
	}

	{
		auto parameter = OP_NumericParameter();
		parameter.name = PARAMETER_KEY_RUNNING;
		parameter.page = page;
		parameter.label = "Running";

		/// We run by default
		parameter.defaultValues[0] = 1;

		const OP_ParAppendResult res = manager->appendToggle(parameter);
		assert(res == OP_ParAppendResult::Success);
	}

	{
		OP_NumericParameter parameter;

		parameter.name = PARAMETER_KEY_FOLD_FRAME_INDEX;
		parameter.page = page;
		parameter.label = "Fold Frame Index";

		parameter.clampMins[0] = true;
		parameter.minValues[0] = 0.0f;
		parameter.defaultValues[0] = 0.0f;

		const OP_ParAppendResult res = manager->appendInt(parameter);
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

	{
		auto parameter = OP_NumericParameter();
		parameter.name = PARAMETER_KEY_TIME_SCALE;
		parameter.page = page;
		parameter.label = "Time Scale";

		parameter.clampMins[0] = true;
		parameter.minValues[0] = 0.0;
		parameter.minSliders[0] = 0.0;

		parameter.maxSliders[0] = 10.0f;

		parameter.defaultValues[0] = 1.0f;

		const OP_ParAppendResult res = manager->appendFloat(parameter, 1);
		assert(res == OP_ParAppendResult::Success);
	}

	{
		auto parameter = OP_NumericParameter();
		parameter.name = PARAMETER_KEY_ADAPTIVE;
		parameter.page = page;
		parameter.label = "Adaptive";

		parameter.defaultValues[0] = 0;

		const OP_ParAppendResult res = manager->appendToggle(parameter);
		assert(res == OP_ParAppendResult::Success);
	}

	{
		auto parameter = OP_NumericParameter();
		parameter.name = PARAMETER_KEY_FRAME_BUDGET;
		parameter.page = page;
		parameter.label = "Frame Budget";

		parameter.clampMins[0] = true;
		parameter.minValues[0] = 0.0;
		parameter.minSliders[0] = 0.0;

		parameter.clampMaxes[0] = true;
		parameter.maxValues[0] = 1.0;
		parameter.maxSliders[0] = 1.0f;

		parameter.defaultValues[0] = 1.0f;

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
	(void)infoSize;
	return false;
}

void Simulator::getInfoDATEntries(int32_t index,
								  int32_t nEntries,
								  TD::OP_InfoDATEntries* entries) {
	(void)index;
	(void)nEntries;
	(void)entries;
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

	// Let's take a look at the timings as well
	OP_TimeInfo const* timeInfo = inputs->getTimeInfo();
	double const rate = timeInfo->rate; // fps

#define update(name, value) .name = cachedInput.name.update(value)

	rtori::rtori_td::Input input = {
	  .inputNumber = cachedInput.inputNumber,
	  update(foldFileSource, std::string(inputs->getParString(PARAMETER_KEY_FOLD_SOURCE))),
	  update(frameIndex, inputs->getParInt(PARAMETER_KEY_FOLD_FRAME_INDEX)),
	  update(foldPercentage,
			 static_cast<float>(inputs->getParDouble(PARAMETER_KEY_FOLD_PERCENTAGE))),

	  update(extractPosition, interests.position),
	  update(extractError, interests.error),
	  update(extractVelocity, interests.velocity),

	  update(timeScale, static_cast<float>(inputs->getParDouble(PARAMETER_KEY_TIME_SCALE))),
	  update(adaptive, inputs->getParInt(PARAMETER_KEY_ADAPTIVE) == 0 ? false : true),
	  update(frameBudget, static_cast<float>(inputs->getParDouble(PARAMETER_KEY_FRAME_BUDGET))),

	  update(targetPeriod,
			 std::chrono::microseconds(static_cast<int64_t>((1000.0 * 1000.0) / rate)))

	};

#undef update

	if (input.changed()) {
		input.inputNumber += 1;
	}

	return input;
}
