#include "Allocator.hpp"

#include <cstdlib>

using namespace rtori::rtori_td::alloc;

void initialize(void** alloc_ctx) {
	*alloc_ctx = nullptr;
}

void deinitialize(void* alloc_ctx) {
	(void)alloc_ctx;
}

void* alloc(const void* alloc_ctx, size_t size, size_t alignment) {
	(void)alloc_ctx;

#ifdef _MSC_VER
	return _aligned_malloc(size, alignment);
#elif __STDC_VERSION__ >= 201112L
	return std::aligned_alloc(alignment, size);
#else
	(void)alignment;
	return std::malloc(size);
#endif
}

void dealloc(const void* dealloc_ctx, void* ptr, size_t size, size_t alignment) {
	(void)dealloc_ctx;

#ifdef _MSC_VER
	(void)size;
	(void)alignment;
	_aligned_free(ptr);
#elif __STDC_VERSION__ >= 202311L
	std::free_aligned_sized(ptr, size, alignment);
#else
	(void)size;
	(void)alignment;
	std::free(ptr);
#endif
}