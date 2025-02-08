#pragma once

#include "rtori_core.hpp"

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
RTORI_TD_EXPORT rtori::Context const* init();
RTORI_TD_EXPORT void deinit(rtori::Context const* ctx);
} // namespace rtori::rtori_td