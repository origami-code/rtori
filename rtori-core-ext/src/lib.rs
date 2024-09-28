#![feature(allocator_api)]

use bitflags::bitflags;
use core::ffi::c_void;
use std::alloc::Allocator;
use std::sync::Arc;
type RawAlloc =
    unsafe extern "C" fn(ctx: *const c_void, size: usize, alignment: usize) -> *mut c_void;
type RawDealloc =
    unsafe extern "C" fn(ctx: *const c_void, ptr: *mut c_void, size: usize, alignment: usize);

#[derive(Clone, Copy, PartialEq)]
#[repr(C)]
pub struct CustomAllocator {
    alloc: RawAlloc,
    dealloc: RawDealloc,
    ctx: *const c_void,
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
            ContextAllocator::Custom(given) => given.deallocate(ptr, layout),
            ContextAllocator::Global => std::alloc::Global.deallocate(ptr, layout),
        }
    }
}

pub struct Context<'alloc> {
    allocator: ContextAllocator<'alloc>,
}

/// Call `rtori_deinit` to deinitialize
#[no_mangle]
pub unsafe extern "C" fn rtori_init(parameters: Option<&CustomAllocator>) -> *const Context<'_> {
    let allocator = parameters
        .map(|custom| ContextAllocator::Custom(custom))
        .unwrap_or(ContextAllocator::Global);

    let context = Context { allocator };

    let boxed = Arc::new_in(context, allocator);

    return Arc::into_raw(boxed);
}

#[no_mangle]
pub unsafe extern "C" fn rtori_clone(ctx: *const Context<'_>) -> *const Context<'_> {
    let ctx = unsafe { Arc::from_raw_in(ctx, (&*ctx).allocator) };
    let cloned = (&ctx).clone();
    std::mem::forget(ctx);
    Arc::into_raw(cloned)
}

#[no_mangle]
pub unsafe extern "C" fn rtori_deinit(ctx: *const Context) {
    let _ctx = unsafe { Arc::from_raw_in(ctx, (&*ctx).allocator) };
    // let it drop naturally
}

#[repr(C)]
pub enum SolverKind {
    /// Origami Simulator by Amanda Ghaessi
    OrigamiSimulator,
}

bitflags! {
    #[repr(C)]
    pub struct BackendFlags: u8 {
        const CPU = 1 << 0;
        const CPU_MT = 1 << 1;

        const GPU_METAL = 1 << 3;
        const GPU_VULKAN = 1 << 4;
        const GPU_DX12 = 1 << 5;
        const GPU_WEBGPU = 1 << 6;

        const GPU_ANY = BackendFlags::GPU_METAL.bits() | BackendFlags::GPU_VULKAN.bits() | BackendFlags::GPU_DX12.bits() | BackendFlags::GPU_WEBGPU.bits();
        const ANY = BackendFlags::GPU_ANY.bits() | BackendFlags::CPU.bits() | BackendFlags::CPU_MT.bits();
    }
}

#[repr(C)]
pub struct Parameters {
    pub solver: SolverKind,
    pub backend: BackendFlags,
}

impl Parameters {
    pub const fn new() -> Self {
        Self {
            solver: SolverKind::OrigamiSimulator,
            backend: BackendFlags::all(),
        }
    }
}

pub enum SimulationKind {
    OrigamiSimulator(u8 /* todo */),
}

pub struct Sim<'alloc> {
    context: Arc<Context<'alloc>>,
    kind: SimulationKind,
}

/// `rtori_sim_init` creates a simulator
#[no_mangle]
pub unsafe extern "C" fn rtori_sim_create<'alloc>(
    ctx: *const Context<'alloc>,
    parameters: Option<&Parameters>,
) -> *mut Sim<'alloc> {
    // Do something
    unimplemented!()
}

/*
pub extern "C" fn rtori_sim_configure(
    sim
)

*/
