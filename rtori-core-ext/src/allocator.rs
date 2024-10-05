use bitflags::bitflags;
use core::ffi::c_void;
use rtori_core::model;
use std::alloc::Allocator;

/// cbindgen:prefix=RTORI_ALLOC_SIZED(2)
type RawAlloc =
    unsafe extern "C" fn(ctx: *const c_void, size: usize, alignment: usize) -> *mut c_void;
type RawDealloc =
    unsafe extern "C" fn(ctx: *const c_void, ptr: *mut c_void, size: usize, alignment: usize);

#[derive(Clone, Copy)]
#[repr(C)]
pub struct Version {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
}

/// cbindgen:ignore
const VERSION: Version = Version {
    major: pkg_version::pkg_version_major!(),
    minor: pkg_version::pkg_version_minor!(),
    patch: pkg_version::pkg_version_patch!(),
};

#[no_mangle]
pub extern "C" fn rtori_version() -> Version {
    VERSION
}

/// The two function pointed to must be thread-safe
#[derive(Clone, Copy, PartialEq)]
#[repr(C)]
pub struct CustomAllocator {
    pub alloc: RawAlloc,
    pub dealloc: RawDealloc,
    pub ctx: *const c_void,
}

unsafe impl<'alloc> Allocator for &'alloc CustomAllocator {
    fn allocate(
        &self,
        layout: std::alloc::Layout,
    ) -> Result<std::ptr::NonNull<[u8]>, std::alloc::AllocError> {
        if layout.size() == 0 {
            return Err(std::alloc::AllocError);
        }

        let ptr = unsafe { (self.alloc)(self.ctx, layout.size(), layout.align()) }.cast::<u8>();

        let ptr_nonnull = std::ptr::NonNull::new(ptr).ok_or(std::alloc::AllocError)?;

        return Ok(std::ptr::NonNull::slice_from_raw_parts(
            ptr_nonnull,
            layout.size(),
        ));
    }

    unsafe fn deallocate(&self, ptr: std::ptr::NonNull<u8>, layout: std::alloc::Layout) {
        if layout.size() == 0 {
            panic!("Should not happen to deallocate a ZST when we should panic on alloc");
        }

        unsafe {
            (self.dealloc)(
                self.ctx,
                ptr.as_ptr().cast::<c_void>(),
                layout.size(),
                layout.align(),
            )
        };
    }
}

/// cbindgen:ignore
#[derive(Clone, Copy, PartialEq)]
pub enum ContextAllocator<'alloc> {
    Custom(&'alloc CustomAllocator),
    Global,
}

unsafe impl<'alloc> Allocator for ContextAllocator<'alloc> {
    fn allocate(
        &self,
        layout: std::alloc::Layout,
    ) -> Result<std::ptr::NonNull<[u8]>, std::alloc::AllocError> {
        match &self {
            ContextAllocator::Custom(given) => given.allocate(layout),
            ContextAllocator::Global => std::alloc::Global.allocate(layout),
        }
    }

    unsafe fn deallocate(&self, ptr: std::ptr::NonNull<u8>, layout: std::alloc::Layout) {
        match &self {
            // SAFETY: as long as the invariants are handled by the C code, this is fine
            ContextAllocator::Custom(given) => unsafe { given.deallocate(ptr, layout) },
            // SAFETY: just a passthrough
            ContextAllocator::Global => unsafe { std::alloc::Global.deallocate(ptr, layout) },
        }
    }
}
