#[cfg(feature = "cpu")]
use rtori_core_simd as os_cpu;
#[cfg(feature = "gpu")]
use rtori_core_wgpu as os_wgpu;

use bitflags::bitflags;

bitflags! {
    #[derive(Copy, Clone, Debug, PartialEq, PartialOrd)]
    #[repr(C)]
    pub struct BackendFlags: u8 {
        const CPU = 1 << 0;

        const GPU_METAL = 1 << 3;
        const GPU_VULKAN = 1 << 4;
        const GPU_DX12 = 1 << 5;
        const GPU_WEBGPU = 1 << 6;

        const GPU_ANY = BackendFlags::GPU_METAL.bits() | BackendFlags::GPU_VULKAN.bits() | BackendFlags::GPU_DX12.bits() | BackendFlags::GPU_WEBGPU.bits();
        const ANY = BackendFlags::GPU_ANY.bits() | BackendFlags::CPU.bits();
    }
}

impl Default for BackendFlags {
    fn default() -> Self {
        BackendFlags::ANY
    }
}

#[derive(Debug)]
pub enum Solver {
    #[cfg(feature = "cpu")]
    CPU(
        Option<
            os_cpu::owned::OwnedRunner<
                'static,
                'static,
                { os_cpu::PREFERRED_WIDTH },
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

    pub fn load_fold(&mut self, fold: &fold::FrameCore) {
        let allocator = alloc::alloc::Global;

        let transformed = rtori_os_fold_importer::transform::transform_in(fold, allocator)
            .expect("Transformation into importation input failed");

        let transformed_input = transformed.with_fold(fold);

        use rtori_os_model::LoaderDyn;
        match self {
            Self::CPU(runner) => {
                let preprocessed =
                    rtori_os_fold_importer::preprocess(&transformed_input, allocator).unwrap();
                let size = preprocessed.size();
                let mut owned_runner = os_cpu::owned::OwnedRunner::with_size(&size);
                {
                    let runner = &mut owned_runner.runner;
                    let mut loader = os_cpu::Loader::new(runner);
                    rtori_os_fold_importer::import_preprocessed_in(
                        &mut loader,
                        &preprocessed,
                        Default::default(),
                        allocator,
                    )
                    .unwrap();
                }
                *runner = Some(owned_runner);
            }
        };
    }

    pub fn step(&mut self, step_count: usize) {
        match self {
            Self::CPU(runner) => {
                let runner = runner.as_mut().unwrap();
                (0..step_count).for_each(|_| runner.step().unwrap())
            }
        }
    }

    /*
    pub fn extract(&self) -> impl rtori_os_model::Extractor<'_> {
        todo!()
    }*/
}
