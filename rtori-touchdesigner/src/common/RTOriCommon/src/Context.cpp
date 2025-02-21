

#include <memory>
#include <rtori/Context.hpp>

#include "rtori/td/Context.hpp"

/*
 * Because the default constructor is constexpr, static std::weak_ptrs are initialized as part
 * of static non-local initialization, before any dynamic non-local initialization begins. This
 * makes it safe to use a std::weak_ptr in a constructor of any static object.
 */
static std::weak_ptr<rtori::Context> sharedContext;

std::shared_ptr<rtori::Context> getContext(void) {
	auto acquired = sharedContext.lock();
	if (acquired.use_count() > 0) {
		return acquired;
	}

	// Here we create a new context
	auto shared = std::make_shared(rtori::Context::global());
	sharedContext = std::weak_ptr(shared);
	return shared;
}