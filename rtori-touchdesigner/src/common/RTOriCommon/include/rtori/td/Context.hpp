#pragma once

#include "rtori_core.hpp"

#ifdef RTORI_TD_BUILD_SHARED
#define RTORI_TD_EXPORT __declspec(dllexport)
#else
#define RTORI_TD_EXPORT
#endif

namespace rtori::rtori_td {
RTORI_TD_EXPORT rtori::Context const* init();
RTORI_TD_EXPORT void deinit(rtori::Context const* ctx);
} // namespace rtori::rtori_td