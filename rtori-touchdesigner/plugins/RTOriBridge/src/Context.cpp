#include "rtori/td/Context.hpp"
#include "rtori_core.hpp"

#include <mutex>
#include <cstdlib>
#include <cassert>

static rtori::Context const* staticRtoriContext = nullptr;
static uint32_t instances = 0;

static void* rtori_alloc(const void* alloc_ctx, size_t size, size_t alignment) {
#ifdef _MSC_VER
	return _aligned_malloc(size, alignment);
#else
	return std::aligned_alloc(alignment, size);
#endif
}

static void rtori_dealloc(const void* dealloc_ctx, void* ptr, size_t size, size_t alignment) {
#ifdef _MSC_VER
	(void)size;
	(void)alignment;
	_aligned_free(ptr);
#else
#if __STDC_VERSION__ >= 202311L
	std::free_aligned_sized(ptr, size, alignment);
#else
	(void)size;
	(void)alignment;
	std::free(ptr);
#endif
#endif
}

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