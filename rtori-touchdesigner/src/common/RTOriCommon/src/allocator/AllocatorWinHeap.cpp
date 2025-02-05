#include "Allocator.hpp"

#ifndef _WIN32
#error "WinHeap allocator is only available on windows"
#endif

#ifndef WIN_HEAP_MAX_ALLOC
#define WIN_HEAP_MAX_ALLOC 0
#endif

#define WIN32_LEAN_AND_MEAN
#include "Windows.h"

using namespace rtori::rtori_td::alloc;

bool initialize(void** alloc_ctx) {
	HANDLE const heap = HeapCreate(0, 0, WIN_HEAP_MAX_ALLOC);
	if (heap == NULL) {
		return false;
	}

	*alloc_ctx = heap;
	return true;
}

void deinitialize(void* alloc_ctx) {
	HANDLE heap = static_cast<HANDLE>(alloc_ctx);
	BOOL const res = HeapDestroy(heap);
	if (res != 0) {
		const DWORD errorCode = GetLastError();
		// TODO: Manage error with FormatMessage
	}
}

void* alloc(void* alloc_ctx, size_t size, size_t alignment) {
	if (MEMORY_ALLOCATION_ALIGNMENT % alignment != 0) {
		return NULL;
	}

	HANDLE heap = static_cast<HANDLE>(alloc_ctx);
	LPVOID allocated = HeapAlloc(heap, 0, size);
	return allocated;
}

void dealloc(void* dealloc_ctx, void* ptr, size_t size, size_t alignment) {
	HANDLE heap = static_cast<HANDLE>(dealloc_ctx);
	BOOL const res = HeapFree(heap, 0, ptr);
	if (res != 0) {
		const DWORD errorCode = GetLastError();
		// TODO: Manage error with FormatMessage
	}
}