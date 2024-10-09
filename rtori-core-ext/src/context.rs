use bitflags::bitflags;

pub struct Context<'alloc> {
    pub(crate) allocator: crate::ContextAllocator<'alloc>,
}

/// Call `rtori_deinit` to deinitialize
#[no_mangle]
pub unsafe extern "C" fn rtori_ctx_init(
    parameters: Option<&crate::CustomAllocator>,
) -> *const Context<'_> {
    let allocator = parameters
        .map(|custom| crate::ContextAllocator::Custom(custom))
        .unwrap_or(crate::ContextAllocator::Global);

    let context = Context { allocator };

    let boxed = crate::Arc::new_in(context, allocator);

    return crate::Arc::into_raw(boxed);
}

#[no_mangle]
pub unsafe extern "C" fn rtori_ctx_clone(ctx: *const Context<'_>) -> *const Context<'_> {
    let ctx = unsafe { crate::Arc::from_raw_in(ctx, (&*ctx).allocator) };
    let cloned = (&ctx).clone();
    std::mem::forget(ctx);
    crate::Arc::into_raw(cloned)
}

#[no_mangle]
pub unsafe extern "C" fn rtori_ctx_deinit(ctx: *const Context) {
    let _ctx = unsafe { crate::Arc::from_raw_in(ctx, (&*ctx).allocator) };
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

/// `rtori_ctx_create_solver` creates a simulator
#[no_mangle]
pub unsafe extern "C" fn rtori_ctx_create_solver<'alloc>(
    ctx: *const Context<'alloc>,
    parameters: *const Parameters,
) -> *const crate::Solver<'alloc> {
    let ctx = {
        let ctx = unsafe { crate::Arc::from_raw_in(ctx, (&*ctx).allocator) };
        let cloned = (&ctx).clone();
        cloned
    };
    let allocator = ctx.allocator;

    use pollster::FutureExt as _;
    let mut solver =
        rtori_core::os_solver::Solver::create(rtori_core::os_solver::BackendFlags::CPU)
            .block_on()
            .unwrap();

    let solver_wrapper = crate::Solver {
        ctx,
        inner: std::sync::Mutex::new(crate::SolverInner {
            solver,
            //state: crate::SolverState::Standby,
        }),
    };

    let arcd = crate::Arc::new_in(solver_wrapper, allocator);
    crate::Arc::into_raw(arcd)
}
