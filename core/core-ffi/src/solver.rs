impl ffi::SolverOperationError {
    pub const fn to_str(&self) -> &'static str {
        match self {
	    &Self::NotLoaded =>
	        "Error(ErrorNotLoaded): Attempted to do an operation requiring that a model be loaded, and no model is",

	    &Self::Extracting =>
		    "Error(ErrorExtracting): Attempted to do an operation that can only be done in the 'Standby' or 'Loaded' state, while it was in the 'Extracting' state",

        &Self::Other => "Error(Other): Other error"
        }
    }

    pub fn format_common<W: core::fmt::Write>(&self, mut f: W) -> core::fmt::Result {
        write!(f, "{}", self.to_str())
    }
}

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

    impl SolverOperationError {
        #[diplomat::attr(auto, stringifier)]
        pub fn format(&self, out: &mut DiplomatWrite) {
            self.format_common(out).unwrap()
        }
    }

    #[derive(Debug)]
    #[repr(C)]
    pub enum SolverLoadError {
        NoSuchFrame,
    }

    #[repr(C)]
    pub enum SolverFamily {
        /// Origami Simulator by Amanda Ghaessi
        OrigamiSimulator,
    }

    /// A Solver is an abstraction over the different solver than can be supported
    #[diplomat::opaque]
    #[derive(Debug)]
    pub struct Solver<'ctx> {
        pub(crate) ctx: &'ctx context::Context<'ctx>,
        pub(crate) inner: rtori_core::Solver<'ctx, alloc::alloc::Global>,
    }

    impl<'ctx> Solver<'ctx> {
        #[diplomat::attr(auto, getter("family"))]
        pub fn family(&self) -> SolverFamily {
            todo!()
        }

        #[diplomat::attr(auto, getter("context"))]
        pub fn context(&self) -> &'ctx context::Context<'ctx> {
            self.ctx
        }

        /// Load from a fold file & frame index
        pub fn load_from_fold(
            &mut self,
            fold: &fold_ffi::FoldFile,
            frame_index: u16,
        ) -> Result<(), SolverLoadError> {
            let frame = fold.inner.borrow_dependent().frame(frame_index);
            match frame {
                Some(frame) => {
                    self.inner.load_fold_in(&frame.resolve());
                    Ok(())
                }
                None => Err(SolverLoadError::NoSuchFrame),
            }
        }

        /// For a 'steppable' solver, step the solver.
        /// Some solvers cannot be stepped, as they do not have any intermediary state.
        pub fn step(&mut self, step_count: u32) -> Result<(), SolverOperationError> {
            if let rtori_core::SolverKind::OS(os_solver) = &mut self.inner.inner {
                os_solver.step(step_count).map_err(|e| match e {
                    rtori_core::os_solver::StepError::NotLoaded => SolverOperationError::NotLoaded,
                    _ => SolverOperationError::Other,
                })
            } else {
                Err(SolverOperationError::Other)
            }
        }

        pub fn set_fold_percentage(&mut self, fold: f32) -> Result<(), SolverOperationError> {
            if let rtori_core::SolverKind::OS(os_solver) = &mut self.inner.inner {
                os_solver
                    .set_fold_percentage(fold)
                    .map_err(|_| SolverOperationError::Other)
            } else {
                Err(SolverOperationError::Other)
            }
        }

        pub fn loaded(&self) -> bool {
            self.inner.loaded()
        }
    }

    /* Extraction */

    #[diplomat::opaque]
    pub struct OSExtractBuilder<'a> {
        position: Option<(&'a mut [f32], u32)>,
        velocity: Option<(&'a mut [f32], u32)>,
        error: Option<(&'a mut [f32], u32)>,
    }

    #[repr(C)]
    pub enum OSExtractBuilderError {
        /// The slice length need to be a multiple of 3 for the position & velocity
        SliceLengthNotMultipleOfThree,
    }

    impl<'a> OSExtractBuilder<'a> {
        #[diplomat::attr(auto, constructor)]
        pub fn new() -> Box<Self> {
            Box::new(Self {
                position: None,
                velocity: None,
                error: None,
            })
        }

        /// Configures a destination for the position buffer, made up of 3-component vectors [x,y,z]
        /// The destination buffer's length must be a multiple of three
        pub fn position(
            &mut self,
            dest: &'a mut [f32],
            offset: u32,
        ) -> Result<(), OSExtractBuilderError> {
            if dest.len() % 3 != 0 {
                Err(OSExtractBuilderError::SliceLengthNotMultipleOfThree)
            } else {
                self.position = Some((dest, offset));
                Ok(())
            }
        }

        /// Do not extract position (like by default)
        pub fn no_position(&mut self) {
            self.position = None;
        }

        /// Configures a destination for the velocity buffer, made up of 3-component vectors [x,y,z]
        /// The destination buffer's length must be a multiple of three
        pub fn velocity(
            &mut self,
            dest: &'a mut [f32],
            offset: u32,
        ) -> Result<(), OSExtractBuilderError> {
            if dest.len() % 3 != 0 {
                Err(OSExtractBuilderError::SliceLengthNotMultipleOfThree)
            } else {
                self.velocity = Some((dest, offset));
                Ok(())
            }
        }

        /// Do not extract velocity (like by default)
        pub fn no_velocity(&mut self) {
            self.velocity = None;
        }

        /// Configures a destination for the error buffer, made up of single floats encoding the error value
        pub fn error(
            &mut self,
            dest: &'a mut [f32],
            offset: u32,
        ) -> Result<(), OSExtractBuilderError> {
            self.error = Some((dest, offset));
            Ok(())
        }

        /// Do not extract error (like by default)
        pub fn no_error(&mut self) {
            self.error = None;
        }
    }

    impl<'ctx> Solver<'ctx> {
        /// Extracts the current state of the solver to the configured builder
        pub fn extract<'a>(&self, request: &mut OSExtractBuilder<'a>) {
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
                let os_solver = if let rtori_core::SolverKind::OS(os_solver) = &self.inner.inner {
                    os_solver
                } else {
                    unreachable!("This should have been eliminated");
                };
                let extractor = os_solver.extract(extract_flags).unwrap();

                if let Some((out, offset)) = request.position.as_mut() {
                    let out: &mut [rtori_core::model::Vector3F] = bytemuck::cast_slice_mut(out);

                    extractor.copy_node_position(out, *offset);
                }

                if let Some((out, offset)) = request.velocity.as_mut() {
                    let out: &mut [rtori_core::model::Vector3F] = bytemuck::cast_slice_mut(out);

                    extractor.copy_node_velocity(out, *offset);
                }

                if let Some((out, offset)) = request.error.as_mut() {
                    extractor.copy_node_error(out, *offset);
                }
            }
        }
    }
}
