#ifndef __FoldInSOP__
#define __FoldInSOP__

#ifdef _MSC_VER
#pragma warning(push, 0)
#endif
#include "SOP_CPlusPlusBase.h"
#include "CPlusPlus_Common.h"
#ifdef _MSC_VER
#pragma warning(pop)
#endif

#include "rtori/td/Context.hpp"

#include <cstdint>

namespace rtori::rtori_td {

/// This SOP is a generator and it takes no input, though it does take a lot of parameters
class FoldInSOP final : public TD::SOP_CPlusPlusBase {
  public:
	FoldInSOP(const TD::OP_NodeInfo* info, rtori::Context const* rtoriCtx);
	virtual ~FoldInSOP();

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

  private:
};

} // namespace rtori::rtori_td
#endif // !__FoldInSOP__
