#pragma once

namespace rtori::rtori_td::alloc {
void initialize(void** alloc_ctx);
void deinitialize(void* alloc_ctx);
void* alloc(void* alloc_ctx, size_t size, size_t alignment);
void dealloc(void* alloc_ctx, size_t size, size_t alignment);
}