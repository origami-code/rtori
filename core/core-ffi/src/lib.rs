#![feature(allocator_api)]

extern crate alloc;
use alloc::alloc::Allocator;
use core::ffi::c_void;

#[cfg(feature = "allocator_api")]
mod allocator {
    type RawAlloc =
        unsafe extern "C" fn(ctx: *const c_void, size: usize, alignment: usize) -> *mut c_void;
    type RawDealloc =
        unsafe extern "C" fn(ctx: *const c_void, ptr: *mut c_void, size: usize, alignment: usize);

    /// The two function pointed to must be thread-safe
    #[derive(Debug, Clone, Copy, PartialEq)]
    #[repr(C)]
    pub struct CustomAllocator {
        pub alloc: RawAlloc,
        pub dealloc: RawDealloc,
        pub ctx: *const c_void,
    }

    unsafe impl<'alloc> Allocator for &'alloc CustomAllocator {
        fn allocate(
            &self,
            layout: alloc::Layout,
        ) -> Result<std::ptr::NonNull<[u8]>, alloc::AllocError> {
            if layout.size() == 0 {
                return Err(alloc::AllocError);
            }

            let ptr = unsafe { (self.alloc)(self.ctx, layout.size(), layout.align()) }.cast::<u8>();

            let ptr_nonnull = std::ptr::NonNull::new(ptr).ok_or(alloc::AllocError)?;

            return Ok(std::ptr::NonNull::slice_from_raw_parts(
                ptr_nonnull,
                layout.size(),
            ));
        }

        unsafe fn deallocate(&self, ptr: std::ptr::NonNull<u8>, layout: alloc::Layout) {
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

    #[derive(Debug, Clone, Copy, PartialEq)]
    pub enum ContextAllocator<'alloc> {
        Custom(&'alloc CustomAllocator),
        Global,
    }

    unsafe impl<'alloc> Allocator for ContextAllocator<'alloc> {
        fn allocate(
            &self,
            layout: alloc::Layout,
        ) -> Result<std::ptr::NonNull<[u8]>, alloc::AllocError> {
            match &self {
                ContextAllocator::Custom(given) => given.allocate(layout),
                ContextAllocator::Global => alloc::Global.allocate(layout),
            }
        }

        unsafe fn deallocate(&self, ptr: std::ptr::NonNull<u8>, layout: alloc::Layout) {
            match &self {
                // SAFETY: as long as the invariants are handled by the C code, this is fine
                ContextAllocator::Custom(given) => unsafe { given.deallocate(ptr, layout) },
                // SAFETY: just a passthrough
                ContextAllocator::Global => unsafe { alloc::Global.deallocate(ptr, layout) },
            }
        }
    }

    /// An extension to allow c-language interface to use a custom allocator
    pub extern "C" fn Context_custom(
        custom: std::ptr::NonNull<crate::CustomAllocator>,
    ) -> *const ffi::Context<'static> {
        let allocator = crate::ContextAllocator::Custom(unsafe { &*(custom.as_ref()) });
        let context = ffi::Context { allocator };

        let boxed = Box::new_in(context, allocator);

        return Box::into_raw(boxed);
    }
}

#[cfg(feature = "allocator_api")]
pub type A<'ctx> = std::alloc::Global;

#[cfg(not(feature = "allocator_api"))]
pub type A<'ctx> = std::alloc::Global;

pub mod context;
pub mod fold;
pub mod solver;

#[diplomat::bridge]
#[diplomat::abi_rename = "rtori_{0}"]
#[diplomat::attr(auto, namespace = "rtori")]
pub mod ffi {

    #[diplomat::opaque]
    #[derive(Debug)]
    pub struct SolverInstance {}
}
