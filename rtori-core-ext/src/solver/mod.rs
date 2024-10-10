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
    //pub(crate) state: SolverState,
}

pub struct Solver<'alloc> {
    pub(crate) ctx: crate::Arc<'alloc, crate::Context<'alloc>>,
    pub(crate) inner: std::sync::Mutex<SolverInner>,
}

/// The context pointer returned is only guaranteed to be valid as long as the solver itself hasn't be de-initialized
#[no_mangle]
pub unsafe extern "C" fn rtori_solver_get_context<'alloc>(
    solver: *const Solver<'alloc>,
) -> *const crate::Context<'alloc> {
    unsafe { &*solver }.ctx.as_ref()
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

#[no_mangle]
pub unsafe extern "C" fn rtori_solver_load_from_transformed<'alloc>(
    solver: *const Solver<'alloc>,
    transformed: *mut crate::TransformedData<'_>,
) -> SolverOperationResult {
    let allocator = unsafe { (&*solver).ctx.allocator };
    let solver = unsafe { crate::Arc::from_raw_in(solver, (&*solver).ctx.allocator) };
    let transformed = unsafe { &*transformed };

    let frame = transformed.input.parsed.frame(transformed.frame);
    let res = match frame {
        Some(frame) => {
            let frame = frame.get();
            let transformed_input = &transformed.transform.with_fold(&frame);

            let mut solver = solver.inner.lock().unwrap();

            solver
                .solver
                .load_transformed_in(&transformed_input, allocator);
            SolverOperationResult::Success
        }
        None => SolverOperationResult::ErrorNoSuchFrameInFold,
    };
    std::mem::forget(solver);

    res
}

/// `rtori_solver_load_from_fold` loads the simulator with data from the given `FoldFrame`
#[no_mangle]
pub unsafe extern "C" fn rtori_solver_load_from_fold<'alloc>(
    solver: *const Solver<'alloc>,
    fold: *const crate::FoldFile<'_>,
    frame_index: u16,
) -> SolverOperationResult {
    let allocator = unsafe { (&*solver).ctx.allocator };
    let solver = unsafe { &*solver };
    let fold = unsafe { &*fold };
    let res = {
        let mut solver = solver.inner.lock().unwrap();

        let frame = fold.parsed.frame(frame_index); // TODO: handle None
        match frame {
            Some(frame) => {
                solver.solver.load_fold_in(&frame.get(), allocator);
                SolverOperationResult::Success
            }
            None => SolverOperationResult::ErrorNoSuchFrameInFold,
        }
    };

    res
}

#[no_mangle]
pub unsafe extern "C" fn rtori_solver_step<'alloc>(
    solver: *const Solver<'alloc>,
    step_count: u32,
) -> SolverOperationResult {
    let res = {
        let solver = unsafe { &*solver };
        let res = {
            let mut solver = solver.inner.lock().unwrap();
            solver.solver.step(step_count)
        };
        res
    };

    match res {
        Ok(_) => SolverOperationResult::Success,
        Err(rtori_core::os_solver::StepError::NotLoaded) => SolverOperationResult::ErrorNotLoaded,
        Err(_) => SolverOperationResult::ErrorOther,
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
    pub positions: crate::ArrayOutputVec3F,
    pub velocity: crate::ArrayOutputVec3F,
    pub error: crate::ArrayOutputF32,
}

/// Extract a field to an array
#[no_mangle]
pub unsafe extern "C" fn rtori_extract<'solver, 'result>(
    solver: *const Solver<'solver>,
    request: *const ExtractOutRequest,
) -> SolverOperationResult {
    let solver = unsafe { &*solver };
    let request: &ExtractOutRequest = unsafe { &*request };

    let extract_flags = crate::model::ExtractFlags::from_bits_truncate(
        0 | if request.positions.buffer_size > 0 {
            crate::model::ExtractFlags::POSITION.bits()
        } else {
            0
        } | if request.velocity.buffer_size > 0 {
            crate::model::ExtractFlags::VELOCITY.bits()
        } else {
            0
        } | if request.error.buffer_size > 0 {
            crate::model::ExtractFlags::ERROR.bits()
        } else {
            0
        },
    );

    let res = {
        let solver = solver
            .inner
            .lock()
            .expect("Expected the solver to already be established");
        // Critical section

        use rtori_core::model::ExtractorDyn as _;
        let extractor = solver.solver.extract(extract_flags).unwrap();

        if let Some(out) = request.positions.buffer {
            let mut out =
                core::ptr::NonNull::slice_from_raw_parts(out, request.positions.buffer_size);
            let out: &mut [crate::model::Vector3F] =
                bytemuck::cast_slice_mut(unsafe { out.as_mut() });
            extractor.copy_node_position(out, request.positions.offset as u32);
        }

        if let Some(out) = request.velocity.buffer {
            let mut out =
                core::ptr::NonNull::slice_from_raw_parts(out, request.velocity.buffer_size);
            let out: &mut [crate::model::Vector3F] =
                bytemuck::cast_slice_mut(unsafe { out.as_mut() });
            extractor.copy_node_velocity(out, request.velocity.offset as u32);
        }

        if let Some(out) = request.error.buffer {
            let mut out = core::ptr::NonNull::slice_from_raw_parts(out, request.error.buffer_size);
            let out: &mut [f32] = unsafe { out.as_mut() };
            extractor.copy_node_error(out, request.error.offset as u32);
        }

        SolverOperationResult::Success
    };

    res
}

/// Drops a solver object. After dropping, the pointer is freed and it should not be used anymore.
#[no_mangle]
pub unsafe extern "C" fn rtori_solver_deinit<'alloc>(solver: *const Solver<'alloc>) {
    println!("DEINIT SOLVER");
    let _fold = unsafe { crate::Arc::from_raw_in(solver, (&*solver).ctx.allocator) };
    // let it drop naturally
}
