/* rtori touchdesigner */

#include "rtori/td/SimulationThread.hpp"
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

#include "rtori/td/Context.hpp"
#include "rtori_core.hpp"

#ifdef _MSC_VER
#define MSVC
#endif

#include <optional>
#include <cassert>
#include <format>

using namespace TD;
using namespace rtori::rtori_td;

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
	customInfo.opLabel->setString("RTOri Simulate (SOP)");
	// Will be turned into a 3 letter icon on the nodes
	customInfo.opIcon->setString("ROS");
	customInfo.majorVersion = RTORI_TOUCHDESIGNER_SIMULATE_SOP_VERSION_MAJOR;
	customInfo.minorVersion = RTORI_TOUCHDESIGNER_SIMULATE_SOP_VERSION_MINOR;

	// Information of the author of the node
	customInfo.authorName->setString("Ars Electronica Futurelab");
	customInfo.authorEmail->setString("futurelab@ars.electronica.art");

	// This SOP takes no inputs by parameter (it is a generator)
	customInfo.minInputs = 0;
	customInfo.maxInputs = 0;
}

DLLEXPORT
SOP_CPlusPlusBase* CreateSOPInstance(const OP_NodeInfo* info) {
	const rtori::Context* const rtoriCtx = rtori::rtori_td::init();

	// Return a new instance of your class every time this is called.
	// It will be called once per SOP that is using the .dll
	return new SimulateSOP(info, rtoriCtx);
}

DLLEXPORT
void DestroySOPInstance(SOP_CPlusPlusBase* instance) {
	// Delete the instance here, this will be called when
	// Touch is shutting down, when the SOP using that instance is deleted, or
	// if the SOP loads a different DLL
	SimulateSOP* instanceCasted = static_cast<SimulateSOP*>(instance);
	const rtori::Context* const rtoriCtx = instanceCasted->rtoriCtx;
	delete instanceCasted;

	rtori::rtori_td::deinit(rtoriCtx);
}
}

constexpr const char* PARAMETER_KEY_POSITION = "Extractposition";
constexpr const char* PARAMETER_KEY_ERROR = "Extracterror";
constexpr const char* PARAMETER_KEY_VELOCITY = "Extractvelocity";

// TODO: Do the simulator retrieval magic
SimulateSOP::SimulateSOP(const OP_NodeInfo* _info, rtori::Context const* rtoriCtx)
	: m_simulator(std::make_shared<rtori::rtori_td::Simulator>(rtoriCtx)), rtoriCtx(rtoriCtx){

																		   };

SimulateSOP::~SimulateSOP(){

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
	// TODO: Use params
	const rtori::rtori_td::Interests interests = {
	  inputs->getParInt(PARAMETER_KEY_POSITION) != 0,
	  inputs->getParInt(PARAMETER_KEY_VELOCITY) != 0,
	  inputs->getParInt(PARAMETER_KEY_ERROR) != 0};
	m_simulator->execute(inputs, interests);

	{
		rtori::rtori_td::OutputGuard simulationOutputGuard = this->m_simulator->query();
		rtori::rtori_td::Output const& simulationOutput = simulationOutputGuard.output;

		// Setting the positions
		if (simulationOutput.positions.has_value()) {
			std::tuple<int32_t, int32_t> range = simulationOutput.positions.value();
			assert((void("Range for the positions should divide evenly by 3 (x, y, z)"),
					(std::get<1>(range) - std::get<0>(range)) % 3 == 0));

			for (size_t i = std::get<0>(range); i < std::get<1>(range); i += 3) {
				// Here, copy
				TD::Position position(simulationOutput.backingBuffer[i],
									  simulationOutput.backingBuffer[i + 1],
									  simulationOutput.backingBuffer[i + 2]);
				output->addPoint(position);
			}
		}

		// Setting the error (they are an attribute of nodes as well)
		if (simulationOutput.error.has_value()) {
			SOP_CustomAttribData nodeErrorAttrib{"Error", 1, AttribType::Float};
			nodeErrorAttrib.floatData =
			  const_cast<float*>(simulationOutput.backingBuffer.data() +
								 std::get<0>(simulationOutput.error.value()));
			output->setCustomAttribute(&nodeErrorAttrib, output->getNumPoints());
		}

		// TODO:
		// Cache the indices and do it outside the mutex
		// As they should only change on geometry change
		output->addTriangles(simulationOutput.indices.data(),
							 simulationOutput.indices.size() / 3);
	}

	// Unfortunately, UVs need to be per-vertex, and I haven't found a way to set
	// vertex attributes from an SOP
	// See
	//
	// forum.derivative.ca/t/c-trouble-adding-more-than-1-set-of-uv-coords-using-settexcoord/257147
	//  So we need to duplicate points for each triangle if we want to set UVs
	//  That could be a split mode of that SOP
	//  OR we call python which can do it
}

void SimulateSOP::executeVBO(SOP_VBOOutput* output, const TD::OP_Inputs* inputs, void*) {
	const rtori::rtori_td::Interests interests = {
	  inputs->getParInt(PARAMETER_KEY_POSITION) != 0,
	  inputs->getParInt(PARAMETER_KEY_VELOCITY) != 0,
	  inputs->getParInt(PARAMETER_KEY_ERROR) != 0};
	m_simulator->execute(inputs, interests);
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

	m_simulator->setupParameters(manager);
}

int32_t SimulateSOP::getNumInfoCHOPChans(void* reserved1) {
	return m_simulator->getNumInfoCHOPChans();
}

void SimulateSOP::getInfoCHOPChan(int32_t index, TD::OP_InfoCHOPChan* chan, void* reserved1) {
	m_simulator->getInfoCHOPChan(index, chan);
}

bool SimulateSOP::getInfoDATSize(TD::OP_InfoDATSize* infoSize, void* reserved1) {
	return m_simulator->getInfoDATSize(infoSize);
}

void SimulateSOP::getInfoDATEntries(int32_t index,
									int32_t nEntries,
									TD::OP_InfoDATEntries* entries,
									void* reserved1) {
	return m_simulator->getInfoDATEntries(index, nEntries, entries);
}

void SimulateSOP::getErrorString(TD::OP_String* error, void* reserved1) {
	return m_simulator->getErrorString(error);
}

void SimulateSOP::getInfoPopupString(TD::OP_String* info, void* reserved1) {
	return m_simulator->getInfoPopupString(info);
}

std::shared_ptr<rtori::rtori_td::Simulator> SimulateSOP::simulator(void) {
	return m_simulator;
}
