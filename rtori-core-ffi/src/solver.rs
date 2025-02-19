#[diplomat::bridge]
#[diplomat::abi_rename = "rtori_{0}"]
#[diplomat::attr(auto, namespace = "rtori")] // TODO: ::solver when https://github.com/rust-diplomat/diplomat/issues/591
pub mod ffi {
    use crate::context::ffi as context;
    use crate::fold::ffi as fold_ffi;

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

    #[derive(Debug)]
    #[repr(C)]
    pub enum SolverLoadError {
        NoSuchFrame
    }

    #[diplomat::opaque]
    #[derive(Debug)]
    pub struct Solver<'ctx> {
        pub(crate) ctx: &'ctx context::Context<'ctx>,
        pub(crate) inner: rtori_core::os_solver::Solver,
    }

    impl<'ctx> Solver<'ctx> {
        #[diplomat::attr(auto, getter("context"))]
        pub fn get_context(&self) -> &'ctx context::Context<'ctx> {
            self.ctx
        }

        pub fn load_from_fold(&mut self, fold: &fold_ffi::FoldFile, frame_index: u16) -> Result<(), SolverLoadError> {
            let frame = fold.inner.frame(frame_index);
                match frame {
                Some(frame) => {
                    self.inner.load_fold_in(&frame.get(), self.ctx.allocator);
                    Ok(())
                }
                None => Err(SolverLoadError::NoSuchFrame),
            }
        }

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
        position: Option<(&'a mut [f32], u32)>,
        velocity: Option<(&'a mut [f32], u32)>,
        error: Option<(&'a mut [f32], u32)>,
    }

    #[repr(C)]
    pub enum ExtractBuilderError {
        /// The slice length need to be a multiple of 3 for the position & velocity
        SliceLengthNotMultipleOfThree,
        /// The offset need to be a multiple of 3 for the position & velocity
        OffsetNotMultipleOfThree,
    }

    impl<'a> ExtractBuilder<'a> {
        #[diplomat::attr(auto, constructor)]
        pub fn new() -> Box<Self> {
            Box::new(Self {
                position: None,
                velocity: None,
                error: None,
            })
        }

        pub fn position(
            &mut self,
            dest: &'a mut [f32],
            offset: u32,
        ) -> Result<(), ExtractBuilderError> {
            if dest.len() % 3 != 0 {
                Err(ExtractBuilderError::SliceLengthNotMultipleOfThree)
            } else if offset % 3 != 0 {
                Err(ExtractBuilderError::OffsetNotMultipleOfThree)
            } else {
                self.position = Some((dest, offset));
                Ok(())
            }
        }

        pub fn velocity(
            &mut self,
            dest: &'a mut [f32],
            offset: u32,
        ) -> Result<(), ExtractBuilderError> {
            if dest.len() % 3 != 0 {
                Err(ExtractBuilderError::SliceLengthNotMultipleOfThree)
            } else if offset % 3 != 0 {
                Err(ExtractBuilderError::OffsetNotMultipleOfThree)
            } else {
                self.velocity = Some((dest, offset));
                Ok(())
            }
        }

        pub fn error(
            &mut self,
            dest: &'a mut [f32],
            offset: u32,
        ) -> Result<(), ExtractBuilderError> {
            self.error = Some((dest, offset));
            Ok(())
        }
    }

    impl<'ctx> Solver<'ctx> {
        pub fn extract<'a>(&self, request: &mut ExtractBuilder<'a>) {
            let extract_flags = rtori_core::model::ExtractFlags::from_bits_truncate(
                0 | if request.position.as_ref().map(|p| p.0.len()).unwrap_or(0) > 0 {
                    rtori_core::model::ExtractFlags::POSITION.bits()
                } else {
                    0
                } | if request.velocity.as_ref().map(|p| p.0.len()).unwrap_or(0) > 0 {
                    rtori_core::model::ExtractFlags::VELOCITY.bits()
                } else {
                    0
                } | if request.error.as_ref().map(|p| p.0.len()).unwrap_or(0) > 0 {
                    rtori_core::model::ExtractFlags::ERROR.bits()
                } else {
                    0
                },
            );

            {
                use rtori_core::model::ExtractorDyn as _;
                let extractor = self.inner.extract(extract_flags).unwrap();

                if let Some((out, offset)) = request.position.as_mut() {
                    let out: &mut [rtori_core::model::Vector3F] =
                        bytemuck::cast_slice_mut(out);

                    extractor.copy_node_position(out, *offset);
                }

                 if let Some((out, offset)) = request.velocity.as_mut() {
                    let out: &mut [rtori_core::model::Vector3F] =
                        bytemuck::cast_slice_mut(out);

                    extractor.copy_node_velocity(out, *offset);
                }

                if let Some((out, offset)) = request.error.as_mut() {
                    extractor.copy_node_error(out, *offset);
                }
            }
        }
    }
}
