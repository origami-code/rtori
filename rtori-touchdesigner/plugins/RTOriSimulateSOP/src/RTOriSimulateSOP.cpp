/* rtori touchdesigner */

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

#include <cassert>

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

DLLEXPORT
SOP_CPlusPlusBase* CreateSOPInstance(const OP_NodeInfo* info) {
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
		// TODO: Do work on DLL unload
	}
};
}

constexpr float DEFAULT_IDLE_THRESHOLD = 0.0001f;

SimulateSOP::SimulateSOP(const OP_NodeInfo*) {
	// TODO: create thread
	this->m_simulationThread = std::thread();
};

SimulateSOP::~SimulateSOP() {
	this->m_simulationThreadExitFlag.test_and_set();
	this->m_threadSignaller.notify_all();

	// joining thread - may block
	this->m_simulationThread.join();
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
	// We convert the parameters into an Input
	const Input consolidated = consolidateParameters(inputs);

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
	{
		SOP_CustomAttribData nodeNormalAttrib{normalsCustomAttributeName, 3, AttribType::Float};
		nodeNormalAttrib.floatData = nullptr; // should point to an array of error values
		output->setCustomAttribute(&nodeNormalAttrib, output->getNumPoints());
	}

	// Setting the error (they are an attribute of nodes as well)
	{
		SOP_CustomAttribData nodeErrorAttrib{"Error", 1, AttribType::Float};
		nodeErrorAttrib.floatData = nullptr; // should point to an array of error values
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
		OP_NumericParameter np;

		np.name = "Gpudirect";
		np.label = "GPU Direct";

		const OP_ParAppendResult res = manager->appendToggle(np);
		assert(res == OP_ParAppendResult::Success);
	}

	{
		OP_NumericParameter np;

		np.name = "Simulationmode";
		np.label = "GPU Direct";

		const OP_ParAppendResult res = manager->appendToggle(np);
		assert(res == OP_ParAppendResult::Success);
	}

	{
		OP_StringParameter foldParameter;
		foldParameter.name = "Foldsource";
		foldParameter.label = "Fold Source";

		const OP_ParAppendResult res = manager->appendString(foldParameter);
		assert(res == OP_ParAppendResult::Success);
	}

	{
		OP_NumericParameter extractPointsParameter;
		extractPointsParameter.name = "Extractpoints";
		extractPointsParameter.label = "Extract points";
		extractPointsParameter.defaultValues[0] = 1;

		const OP_ParAppendResult res = manager->appendToggle(extractPointsParameter);
		assert(res == OP_ParAppendResult::Success);
	}

	{
		OP_NumericParameter extractUVsParameter;
		extractUVsParameter.name = "Extractuvs";
		extractUVsParameter.label = "Extract UVs";

		const OP_ParAppendResult res = manager->appendToggle(extractUVsParameter);
		assert(res == OP_ParAppendResult::Success);
	}

	{
		OP_NumericParameter extractErrorParameter;
		extractErrorParameter.name = "Extracterror";
		extractErrorParameter.label = "Extract Error";

		const OP_ParAppendResult res = manager->appendToggle(extractErrorParameter);
		assert(res == OP_ParAppendResult::Success);
	}

	{
		OP_StringParameter outputKindParameter;
		outputKindParameter.name = "Outputkind";
		outputKindParameter.label = "Output Kind ";
		outputKindParameter.defaultValue = "point-per-vertex";

		const char** choices = new const char* [2] { "point-per-node", "point-per-vertex" };
		const char** labels = new const char* [2]
		{
			"One point per node (more efficient - no UVs)",
			  "one point per Vertex (less efficient - UV copied)"
		};

		const OP_ParAppendResult res =
		  manager->appendMenu(outputKindParameter, 2, choices, labels);
		assert(res == OP_ParAppendResult::Success);
	}

	{
		OP_NumericParameter creasePercentageParameter;
		creasePercentageParameter.name = "Creasepercentage";
		creasePercentageParameter.label = "Crease Percentage";

		creasePercentageParameter.clampMins[0] = true;
		creasePercentageParameter.clampMaxes[0] = true;
		creasePercentageParameter.minSliders[0] = -1.0f;
		creasePercentageParameter.maxSliders[0] = 1.0f;
		creasePercentageParameter.defaultValues[0] = 0.0f;

		const OP_ParAppendResult res = manager->appendFloat(creasePercentageParameter, 1);
		assert(res == OP_ParAppendResult::Success);
	}

	{
		auto idleThresholdParameter = OP_NumericParameter();
		idleThresholdParameter.name = "Idlethreshold";
		idleThresholdParameter.label = "Idle threshold",
		idleThresholdParameter.defaultValues[0] = DEFAULT_IDLE_THRESHOLD;
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
	auto input = Input{.fold = inputs->getParString("Foldsource"),
					   .extractPositions = inputs->getParInt("Extractpositions") != 0,
					   .extractError = inputs->getParInt("Extracterror") != 0,
					   .extractVelocity = inputs->getParInt("Extractvelocity") != 0};

	return input;
}