/* rtori touchdesigner */

#include "rtori/td/SimulationThread.hpp"
#include <cassert>

#ifndef RTORI_TD_VERSION_MAJOR
#ifndef _MSC_VER
#warning "RTORI_TD_VERSION_MAJOR undefined, setting to 0"
#else
#pragma warning("RTORI_TD_VERSION_MAJOR undefined, setting to 0")
#endif
#define RTORI_TD_VERSION_MAJOR 0
#endif

#ifndef RTORI_TD_VERSION_MINOR
#ifndef _MSC_VER
#warning "RTORI_TD_VERSION_MINOR undefined, setting to 0"
#else
#pragma warning("RTORI_TD_VERSION_MINOR undefined, setting to 0")
#endif
#define RTORI_TD_VERSION_MINOR 0
#endif

#include "RTOriFoldInSOP.hpp"

#ifdef _MSC_VER
#pragma warning(push, 0)
#endif
#include "SOP_CPlusPlusBase.h"
#include "CPlusPlus_Common.h"
#ifdef _MSC_VER
#pragma warning(pop)
#endif

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
	customInfo.opType->setString("Rtorifoldin");
	// English readable name
	customInfo.opLabel->setString("RTOri Fold In");
	// Will be turned into a 3 letter icon on the nodes
	customInfo.opIcon->setString("Ofi");
	customInfo.majorVersion = RTORI_TD_VERSION_MAJOR;
	customInfo.minorVersion = RTORI_TD_VERSION_MINOR;

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
	return new FoldInSOP(info, rtoriCtx);
}

DLLEXPORT
void DestroySOPInstance(SOP_CPlusPlusBase* instance) {
	// Delete the instance here, this will be called when
	// Touch is shutting down, when the SOP using that instance is deleted, or
	// if the SOP loads a different DLL
	FoldInSOP* instanceCasted = static_cast<FoldInSOP*>(instance);
	const rtori::Context* const rtoriCtx = instanceCasted->rtoriCtx;
	delete instanceCasted;

	rtori::rtori_td::deinit(rtoriCtx);
}
}

constexpr const char* PARAMETER_KEY_SOURCE_INDEX = "Source";
constexpr const char* PARAMETER_KEY_FRAME_INDEX = "Frameindex";

// TODO: Do the simulator retrieval magic
FoldInSOP::FoldInSOP(const OP_NodeInfo* _info, rtori::Context const* rtoriCtx)
	: rtoriCtx(rtoriCtx){

																		   };

FoldInSOP::~FoldInSOP(){

};

void FoldInSOP::getGeneralInfo(SOP_GeneralInfo* ginfo, const TD::OP_Inputs* inputs, void*) {
	// This will cause the node to cook every frame if the output is used
	ginfo->cookEveryFrameIfAsked = true;

	// Direct shape to GPU loading if asked
	ginfo->directToGPU = inputs->getParInt("Gpudirect") != 0 ? true : false;

	// New plugins should always use CCW winding.
	ginfo->winding = SOP_Winding::CCW;
}

void FoldInSOP::execute(SOP_Output* output, const TD::OP_Inputs* inputs, void*) {
	// Unfortunately, UVs need to be per-vertex, and I haven't found a way to set
	// vertex attributes from an SOP
	// See
	//
	// forum.derivative.ca/t/c-trouble-adding-more-than-1-set-of-uv-coords-using-settexcoord/257147
	//  So we need to duplicate points for each triangle if we want to set UVs
	//  That could be a split mode of that SOP
	//  OR we call python which can do it

	// TODO
}

void FoldInSOP::executeVBO(SOP_VBOOutput* output, const TD::OP_Inputs* inputs, void*) {
	// TODO
}

void FoldInSOP::setupParameters(TD::OP_ParameterManager* manager, void*) {
	{
		OP_NumericParameter parameter;

		parameter.name = "Gpudirect";
		parameter.label = "GPU Direct";

		const OP_ParAppendResult res = manager->appendToggle(parameter);
		assert(res == OP_ParAppendResult::Success);
	}

	{
		OP_NumericParameter parameter;
		parameter.name = PARAMETER_KEY_SOURCE_INDEX;
		parameter.label = "Source Index";
		parameter.defaultValues[0] = 0;

		const OP_ParAppendResult res = manager->appendToggle(parameter);
		assert(res == OP_ParAppendResult::Success);
	}

	{
		OP_NumericParameter parameter;
		parameter.name = PARAMETER_KEY_FRAME_INDEX;
		parameter.label = "Frame Index";
		parameter.defaultValues[0] = 0;

		const OP_ParAppendResult res = manager->appendToggle(parameter);
		assert(res == OP_ParAppendResult::Success);
	}
}

int32_t FoldInSOP::getNumInfoCHOPChans(void* reserved1) {
	(void)reserved1;
	return 0;
	// TODO
}

void FoldInSOP::getInfoCHOPChan(int32_t index, TD::OP_InfoCHOPChan* chan, void* reserved1) {
	(void)index;
	(void)chan;
	(void)reserved1;
	// TODO
}

bool FoldInSOP::getInfoDATSize(TD::OP_InfoDATSize* infoSize, void* reserved1) {
	(void)infoSize;
	(void)reserved1;
	return false;
}

void FoldInSOP::getInfoDATEntries(int32_t index,
								  int32_t nEntries,
								  TD::OP_InfoDATEntries* entries,
								  void* reserved1) {
	(void)index;
	(void)nEntries;
	(void)entries;
	(void)reserved1;
	// TODO
}

void FoldInSOP::getErrorString(TD::OP_String* error, void* reserved1) {
	(void)error;
	(void)reserved1;
	// TODO
}

void FoldInSOP::getInfoPopupString(TD::OP_String* info, void* reserved1) {
	(void)info;
	(void)reserved1;
	// TODO
}
