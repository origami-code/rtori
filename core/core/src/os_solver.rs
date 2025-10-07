use core::alloc::Allocator;

#[cfg(feature = "gpu")]
use rtori_core_wgpu as os_wgpu;
#[cfg(feature = "cpu")]
use rtori_os_simd as os_cpu;

use crate::BackendFlags;
pub use rtori_os_fold_importer as fold_importer;

/// Origami based solver
#[derive(Debug)]
pub enum Solver {
    #[cfg(feature = "cpu")]
    CPU(
        Option<
            os_cpu::owned::OwnedRunner<
                'static,
                { os_cpu::MIN_LANE_COUNT_32 },
                alloc::alloc::Global,
            >,
        >,
    ),
    #[cfg(feature = "gpu")]
    GPU(GPURunner),
}

impl Solver {
    pub async fn create(backends: BackendFlags) -> Result<Self, ()> {
        #[cfg(feature = "gpu")]
        if backends.intersects(BackendFlags::GPU_ANY) {
            let wgpu_backends = if backends.contains(BackendFlags::GPU_ANY) {
                os_wgpu::wgpu::Backends::all()
            } else {
                let mut output = os_wgpu::wgpu::Backends::empty();
                #[cfg(feature = "dx12")]
                if backends.contains(BackendFlags::GPU_DX12) {
                    output |= os_wgpu::wgpu::Backends::DX12;
                }
                #[cfg(feature = "vulkan")]
                if backends.contains(BackendFlags::GPU_VULKAN) {
                    output |= os_wgpu::wgpu::Backends::VULKAN;
                }
                #[cfg(feature = "metal")]
                if backends.contains(BackendFlags::GPU_METAL) {
                    output |= os_wgpu::wgpu::Backends::METAL;
                }
                #[cfg(feature = "webgpu")]
                if backends.contains(BackendFlags::GPU_WEBGPU) {
                    output |= os_wgpu::wgpu::Backends::BROWSER_WEBGPU;
                }
                output
            };

            return GPURunner::create(wgpu_backends)
                .await
                .map(|inner| Self::GPU(inner));
        }

        #[cfg(feature = "cpu")]
        if backends.intersects(BackendFlags::CPU) {
            return Ok(Solver::CPU(None));
        }

        // Invalid
        Err(())
    }

    pub fn load_preprocessed_in<I, PA, A>(
        &mut self,
        preprocessed: &fold_importer::InputWithCreaseGeometry<'_, I, PA>,
        allocator: A,
    ) where
        I: fold_importer::input::ImportInput,
        PA: Allocator,
        A: Allocator + Clone,
    {
        match self {
            Self::CPU(runner) => {
                let size = preprocessed.compute_size();
                let mut owned_runner = os_cpu::owned::OwnedRunner::with_size(&size);
                {
                    let runner = owned_runner.runner_mut();
                    let mut loader = os_cpu::Loader::new(runner);
                    preprocessed
                        .load(&mut loader, Default::default(), allocator)
                        .unwrap();
                }
                *runner = Some(owned_runner);
            }
            _ => unimplemented!(),
        };
    }

    pub fn load_transformed_in<IA, A>(
        &mut self,
        transformed: &fold_importer::supplement::SupplementedInput<'_, IA>,
        allocator: A,
    ) where
        IA: Allocator,
        A: Allocator + Clone,
    {
        let preprocessed =
            fold_importer::InputWithCreaseGeometry::process(transformed, allocator.clone())
                .unwrap();

        self.load_preprocessed_in(&preprocessed, allocator)
    }

    pub fn load_fold_in<A: Allocator>(&mut self, fold: &fold::FrameCore, allocator: A)
    where
        A: Allocator + Clone,
    {
        let transformed = rtori_os_fold_importer::supplement::transform_in(fold, allocator.clone())
            .expect("Transformation into importation input failed");

        let transformed_input = transformed.with_fold(fold);

        self.load_transformed_in(&transformed_input, allocator);
    }

    pub fn step(&mut self, step_count: u32) -> Result<(), StepError> {
        match self {
            Self::CPU(runner) => {
                let runner = runner.as_mut().ok_or(StepError::NotLoaded)?;
                (0..step_count).try_for_each(|step_number| {
                    runner.step().map_err(|_| StepError::Other {
                        local_step_number: step_number,
                    })
                })
            }
            _ => unimplemented!(),
        }
    }

    pub fn extract(
        &self,
        extract_flags: rtori_os_model::ExtractFlags,
    ) -> Result<Extractor<'_>, ExtractError> {
        match self {
            Self::CPU(runner) => runner
                .as_ref()
                .ok_or(ExtractError::NotLoaded)
                .map(|runner| Extractor::CPU(runner.extract(extract_flags))),
        }
    }

    pub fn set_fold_percentage(&mut self, fold_percentage: f32) -> Result<(), ()> {
        match self {
            Self::CPU(runner) => runner
                .as_mut()
                .ok_or(())
                .map(|runner| runner.set_fold_percentage(fold_percentage)),
        }
    }

    pub fn loaded(&self) -> bool {
        match self {
            Self::CPU(runner) => runner.is_some(),
            _ => false,
        }
    }

    /*
    pub fn extract(&self) -> impl rtori_os_model::Extractor<'_> {
        todo!()
    }*/
}

pub enum Extractor<'borrow> {
    CPU(rtori_os_simd::Extractor<'borrow, { rtori_os_simd::MIN_LANE_COUNT_32 }>),
}

impl rtori_os_model::ExtractorDyn<'_> for Extractor<'_> {
    fn count_nodes(&self) -> usize {
        match self {
            Self::CPU(inner) => inner.count_nodes(),
        }
    }

    fn copy_node_position(
        &self,
        to: &mut [rtori_os_model::Vector3F],
        from: rtori_os_model::NodeIndex,
    ) -> bool {
        match self {
            Self::CPU(inner) => inner.copy_node_position(to, from),
        }
    }

    fn copy_node_velocity(
        &self,
        to: &mut [rtori_os_model::Vector3F],
        from: rtori_os_model::NodeIndex,
    ) -> bool {
        match self {
            Self::CPU(inner) => inner.copy_node_velocity(to, from),
        }
    }

    fn copy_node_error(&self, to: &mut [f32], from: rtori_os_model::NodeIndex) -> bool {
        match self {
            Self::CPU(inner) => inner.copy_node_error(to, from),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum ExtractError {
    NotLoaded,
}

#[derive(Debug, Clone, Copy)]
pub enum StepError {
    NotLoaded,
    Other { local_step_number: u32 },
}
