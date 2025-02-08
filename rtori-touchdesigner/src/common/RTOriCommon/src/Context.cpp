
#include "rtori/td/Context.hpp"
#include "rtori_core.hpp"

#include "allocator/Allocator.hpp"

#include <mutex>
#include <cstdlib>
#include <cassert>

/// There is a single context, shared between all OPs
static rtori::Context const* staticRtoriContext = nullptr;

/// Reference-counting doesn't need to be thread safe as
/// touchdesigner's model is single-threaded
static uint32_t instances = 0;

namespace rtori::rtori_td {

rtori::Context const* init() {
	uint32_t previousInstanceCount = instances++;
	if (previousInstanceCount == 0) {
		// TODO: use the correct data structure
		staticRtoriContext = rtori::rtori_ctx_init(nullptr);
	}
	assert((void("context should be non-null by this point"), staticRtoriContext != nullptr));
	return staticRtoriContext;
}

void deinit(rtori::Context const* ctx) {
	assert((void("Unknown context given"), ctx == staticRtoriContext));
	assert((void("Instance count should be non-zero"), instances >= 1));

	instances -= 1;

	if (instances == 0) {
		rtori::rtori_ctx_deinit(staticRtoriContext);
		staticRtoriContext = nullptr;
	}
}
} // namespace rtori::rtori_td