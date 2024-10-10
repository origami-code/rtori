#pragma once

#include "rtori_core.hpp"

namespace rtori::rtori_td {
rtori::Context const* init();
void deinit(rtori::Context const* ctx);
} // namespace rtori::rtori_td