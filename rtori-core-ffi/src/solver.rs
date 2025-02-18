#[diplomat::bridge]
#[diplomat::abi_rename = "rtori_{0}"]
#[diplomat::attr(auto, namespace = "rtori")] // TODO: ::solver when https://github.com/rust-diplomat/diplomat/issues/591
pub mod ffi {
    use crate::context::ffi as context;

    #[derive(Debug)]
    #[repr(C)]
    pub enum SolverOperationError {
        /// Attempted to do an operation requiring that a model be loaded, while in "Standby"
        NotLoaded,

        /// Attempted to do an operation that can only be done in the "Standby" or "Loaded" state,
        /// while it was in the "Extracting" state. This means the solver is already mapped.
        Extracting,

        /// Another error
        Other,
    }

    #[diplomat::opaque]
    #[derive(Debug)]
    pub struct Solver<'ctx> {
        ctx: &'ctx context::Context<'ctx>,
        inner: rtori_core::os_solver::Solver,
    }

    impl<'ctx> Solver<'ctx> {
        /// For a 'steppable' solver, step the solver.
        /// Some solvers cannot be stepped, as they do not have any intermediary state.
        pub fn step(&mut self, step_count: u32) -> Result<(), SolverOperationError> {
            self.inner.step(step_count).map_err(|e| match e {
                rtori_core::os_solver::StepError::NotLoaded => SolverOperationError::NotLoaded,
                _ => SolverOperationError::Other,
            })
        }
    }

    /* Extraction */

    #[diplomat::opaque]
    pub struct ExtractBuilder<'a> {
        pub position: Option<(DiplomatSliceMut<'a, f32>, Option<Range>)>,
        pub velocity: Option<(DiplomatSliceMut<'a, f32>, Option<Range>)>,
        pub error: Option<(DiplomatSliceMut<'a, f32>, Option<Range>)>,
    }

    #[repr(C)]
    pub struct Range {
        pub from: usize,
        pub count: usize,
    }

    impl<'a> ExtractBuilder<'a> {
        pub fn position(&mut self, dest: DiplomatSliceMut<'a, f32>, range: Option<Range>) {
            self.position = Some((dest, range));
        }

        pub fn velocity(&mut self, dest: DiplomatSliceMut<'a, f32>, range: Option<Range>) {
            self.velocity = Some((dest, range));
        }

        pub fn error(&mut self, dest: DiplomatSliceMut<'a, f32>, range: Option<Range>) {
            self.error = Some((dest, range));
        }
    }

    impl<'ctx> Solver<'ctx> {
        pub fn extract<'a>(&self, request: &ExtractBuilder<'a>) {
            /*
                    let extract_flags = crate::model::ExtractFlags::from_bits_truncate(
                        0 | if request.position.buffer_size > 0 {
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
            */
            todo!()
        }
    }
}
