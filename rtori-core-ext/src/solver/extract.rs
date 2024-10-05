

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    #[repr(C)]
    pub struct ExtractFlags: u16 {
        const POSITION = 0b0000_0000_0000_0001;
        const ERROR = 0b0000_0000_0000_0010;
        const VELOCITY = 0b0000_0000_0000_0100;
        const FOLD_ANGLE = 0b0000_0000_0000_1000;
        const ALL = Self::POSITION.bits() | Self::ERROR.bits() | Self::VELOCITY.bits() | Self::FOLD_ANGLE.bits();
    }
}



static_assertions::const_assert_eq!(ExtractFlags::POSITION.bits(), model::ExtractFlags::POSITION.bits());
static_assertions::const_assert_eq!(ExtractFlags::ERROR.bits(), model::ExtractFlags::ERROR.bits());
static_assertions::const_assert_eq!(ExtractFlags::VELOCITY.bits(), model::ExtractFlags::VELOCITY.bits());
static_assertions::const_assert_eq!(ExtractFlags::FOLD_ANGLE.bits(), model::ExtractFlags::FOLD_ANGLE.bits());

impl ExtractFlags {
    pub const fn into_model(self) -> model::ExtractFlags {
        crate::model::ExtractFlags::from_bits_truncate(self.bits())
    }
}

pub struct ExtractWrapper<'alloc> {
    solver: crate::Arc<'alloc, Solver<'alloc>>,
    extractor: rtori_core::os_solver::Extractor<'alloc>
}

#[repr(C)]
pub struct ExtractResult<'alloc> {
    pub kind: SolverOperationResult,
    pub data: Option<*const ExtractWrapper<'alloc>>
}

#[no_mangle]
pub unsafe extern "C" fn rtori_extract<'solver, 'result>(
    solver: *const Solver<'solver>,
    extract_flags: ExtractFlags
) -> ExtractResult<'result> {

    let solver = unsafe { crate::Arc::from_raw_in(solver, (&*solver).ctx.allocator) };

    let mut solver = Some(solver);
    
    let res = {
        let mut solver = solver.as_ref().unwrap().inner.lock().unwrap();
        match solver.state {
            SolverState::Extracting => None,
            _ => Some({
                solver.state = SolverState::Extracting;

                solver.solver.extract(extract_flags.into_model())
                
            })
        }
        
    };
    
    let res = match res {
        Some(Err(e)) => ExtractResult {
            kind: SolverOperationResult::ErrorOther,
            data: None
        },
        Some(Ok(e)) => ExtractResult {
            kind: SolverOperationResult::Success,
            data: {
                let solver = solver.take().unwrap();

                let arc = crate::Arc::new_in({
                    ExtractWrapper {
                        solver,
                        extractor: e
                    }
                }, todo!());

                Some(crate::Arc::into_raw(arc))
            }
        },
        None => ExtractResult {
            kind: SolverOperationResult::ErrorExtracting,
            data: None
        },
    };
    
    // We only forget it if we've used it 
    if let Some(solver) = solver.take() {
        std::mem::forget(solver)
    };

    res
    
}