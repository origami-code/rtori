use bitflags::bitflags;
use core::ffi::c_void;
use rtori_core::model;
use std::alloc::Allocator;

#[repr(u8)]
pub enum SolverState {
    Standby,
    Loaded,
    Extracting,
}

pub(crate) struct SolverInner {
    pub(crate) solver: rtori_core::os_solver::Solver,
    pub(crate) state: SolverState,
}

pub struct Solver<'alloc> {
    pub(crate) ctx: crate::Arc<'alloc, crate::Context<'alloc>>,
    pub(crate) inner: std::sync::Mutex<SolverInner>,
}

#[derive(Debug, Clone, Copy)]
#[repr(u32)]
pub enum SolverOperationResult {
    Success = 0x00,
    /// Attempted to do an operation requiring that a model be loaded, while in "Standby"
    ErrorNotLoaded,
    /// Attempted to do an operation that can only be done in the "Standby" or "Loaded" state, while it was in the "Extracting" state
    ErrorExtracting,
    ErrorNoSuchFrameInFold,
    ErrorOther = 0xFF,
}

/// `rtori_solver_load_from_fold` loads the simulator with data from the given `FoldFrame`
#[no_mangle]
pub unsafe extern "C" fn rtori_solver_load_from_fold<'alloc>(
    solver: *const Solver<'alloc>,
    fold: *const crate::FoldFile<'_>,
    frame_index: u16,
) -> SolverOperationResult {
    let allocator = unsafe { (&*solver).ctx.allocator };
    let solver = unsafe { crate::Arc::from_raw_in(solver, (&*solver).ctx.allocator) };
    let fold = unsafe { crate::Arc::from_raw_in(fold, (&*fold).ctx.allocator) };
    let res = {
        let mut solver = solver.inner.lock().unwrap();
        if matches!(solver.state, SolverState::Extracting) {
            SolverOperationResult::ErrorExtracting
        } else {
            let frame = fold.parsed.frame(frame_index); // TODO: handle None
            match frame {
                Some(frame) => {
                    solver.solver.load_fold_in(&frame.get(), allocator);
                    SolverOperationResult::Success
                }
                None => SolverOperationResult::ErrorNoSuchFrameInFold,
            }
        }
    };
    std::mem::forget(solver);
    std::mem::forget(fold);

    res
}

#[no_mangle]
pub unsafe extern "C" fn rtori_solver_step<'alloc>(
    solver: *const Solver<'alloc>,
    step_count: u32,
) -> SolverOperationResult {
    let res = {
        let solver = unsafe { crate::Arc::from_raw_in(solver, (&*solver).ctx.allocator) };
        let res = {
            let mut solver = solver.inner.lock().unwrap();
            match solver.state {
                SolverState::Extracting => None,
                _ => Some(solver.solver.step(step_count)),
            }
        };
        std::mem::forget(solver);
        res
    };

    match res {
        None => SolverOperationResult::ErrorExtracting,
        Some(Ok(_)) => SolverOperationResult::Success,
        Some(Err(rtori_core::os_solver::StepError::NotLoaded)) => {
            SolverOperationResult::ErrorNotLoaded
        }
        Some(Err(_)) => SolverOperationResult::ErrorOther,
    }
}

#[repr(C)]
pub struct ExtractOutRange {
    pub offset: usize,

    /// The connected array must be able to contain the given amount of items
    pub item_count: usize,
}

#[repr(C)]
pub struct ExtractOutRequest {
    pub positions: Option<core::ptr::NonNull<[f32; 3]>>,
    pub positions_range: ExtractOutRange,

    pub velocity: Option<core::ptr::NonNull<[f32; 3]>>,
    pub velocity_range: ExtractOutRange,

    pub error: Option<core::ptr::NonNull<f32>>,
    pub error_range: ExtractOutRange,
}

/// Extract a field to an array
#[no_mangle]
pub unsafe extern "C" fn rtori_extract<'solver, 'result>(
    solver: *const Solver<'solver>,
    request: *const ExtractOutRequest,
) -> SolverOperationResult {
    let solver = unsafe { crate::Arc::from_raw_in(solver, (&*solver).ctx.allocator) };
    let request: &ExtractOutRequest = unsafe { &*request };

    let extract_flags = crate::model::ExtractFlags::from_bits_truncate(
        0 | if request.positions.is_some() && request.positions_range.item_count > 0 {
            crate::model::ExtractFlags::POSITION.bits()
        } else {
            0
        } | if request.velocity.is_some() && request.velocity_range.item_count > 0 {
            crate::model::ExtractFlags::VELOCITY.bits()
        } else {
            0
        } | if request.error.is_some() && request.error_range.item_count > 0 {
            crate::model::ExtractFlags::ERROR.bits()
        } else {
            0
        },
    );

    let res = {
        let mut solver = solver.inner.lock().unwrap();
        // Critical section
        match solver.state {
            SolverState::Extracting => SolverOperationResult::ErrorExtracting,
            _ => {
                use rtori_core::model::ExtractorDyn as _;
                solver.state = SolverState::Extracting;
                let extractor = solver.solver.extract(extract_flags).unwrap();

                if let Some(out) = request.positions {
                    let mut out = core::ptr::NonNull::slice_from_raw_parts(
                        out,
                        request.positions_range.item_count,
                    );
                    let out: &mut [crate::model::Vector3F] =
                        bytemuck::cast_slice_mut(unsafe { out.as_mut() });
                    extractor.copy_node_position(out, request.positions_range.offset as u32);
                }

                if let Some(out) = request.velocity {
                    let mut out = core::ptr::NonNull::slice_from_raw_parts(
                        out,
                        request.velocity_range.item_count,
                    );
                    let out: &mut [crate::model::Vector3F] =
                        bytemuck::cast_slice_mut(unsafe { out.as_mut() });
                    extractor.copy_node_velocity(out, request.velocity_range.offset as u32);
                }

                if let Some(out) = request.error {
                    let mut out = core::ptr::NonNull::slice_from_raw_parts(
                        out,
                        request.error_range.item_count,
                    );
                    let out: &mut [f32] = unsafe { out.as_mut() };
                    extractor.copy_node_error(out, request.error_range.offset as u32);
                }

                SolverOperationResult::Success
            }
        }
    };

    std::mem::forget(solver);

    res
}

/// Drops a solver object. After dropping, the pointer is freed and it should not be used anymore.
#[no_mangle]
pub unsafe extern "C" fn rtori_solver_deinit<'alloc>(solver: *mut Solver<'alloc>) {
    let _fold = unsafe { crate::Arc::from_raw_in(solver, (&*solver).ctx.allocator) };
    // let it drop naturally
}
