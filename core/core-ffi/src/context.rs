#[diplomat::bridge]
#[diplomat::abi_rename = "rtori_{0}"]
#[diplomat::attr(auto, namespace = "rtori")]
pub mod ffi {

    /// A context is an allocation arena, which can be unallocated at any point.
    /// There may be several in a process.
    #[diplomat::opaque]
    #[derive(Debug)]
    pub struct Context<'alloc> {
        pub(crate) allocator: crate::A<'alloc>,
        _marker: core::marker::PhantomData<&'alloc crate::A<'alloc>>,
    }

    impl<'a> Context<'a> {
        #[diplomat::attr(auto, constructor)]
        pub const fn global() -> Box<Self, crate::A<'a>> {
            todo!()
        }
    }

    #[repr(C)]
    pub enum Backend {
        CPU,
        CPUMt,
        GPUMetal,
        GPUVulkan,
        GPUDX12,
        GPUWEBGPU,
    }

    #[repr(C)]
    pub struct BackendFlags {
        pub value: u8,
    }

    impl BackendFlags {
        pub const CPU: Self = Self { value: 1 << 0 };
        pub const CPU_MT: Self = Self { value: 1 << 1 };

        pub const GPU_METAL: Self = Self { value: 1 << 3 };
        pub const GPU_VULKAN: Self = Self { value: 1 << 4 };
        pub const GPU_DX12: Self = Self { value: 1 << 5 };
        pub const GPU_WEBGPU: Self = Self { value: 1 << 6 };

        pub fn gpu_metal() -> Self {
            Self {
                value: Self::GPU_METAL.value,
            }
        }

        pub fn gpu_vulkan() -> Self {
            Self {
                value: Self::GPU_VULKAN.value,
            }
        }

        pub fn gpu_dx12() -> Self {
            Self {
                value: Self::GPU_DX12.value,
            }
        }

        pub fn gpu_any() -> Self {
            Self {
                value: Self::GPU_METAL.value
                    | Self::GPU_VULKAN.value
                    | Self::GPU_DX12.value
                    | Self::GPU_WEBGPU.value,
            }
        }

        pub fn cpu() -> Self {
            Self {
                value: Self::CPU.value,
            }
        }

        pub fn cpu_mt() -> Self {
            Self {
                value: Self::CPU_MT.value,
            }
        }

        pub fn cpu_any() -> Self {
            Self {
                value: Self::CPU.value | Self::CPU_MT.value,
            }
        }

        #[diplomat::attr(auto, constructor)]
        pub fn any() -> Self {
            Self {
                value: Self::cpu_any().value | Self::gpu_any().value,
            }
        }
    }

    #[repr(C)]
    pub enum SolverFamily {
        /// Origami Simulator by Amanda Ghaessi
        OrigamiSimulator,
    }

    #[repr(C)]
    pub struct Parameters {
        pub family: SolverFamily,
        /// Acceptable backends
        pub backend: BackendFlags,
    }

    impl Parameters {
        #[diplomat::attr(auto, constructor)]
        pub fn new() -> Self {
            Self {
                family: SolverFamily::OrigamiSimulator,
                backend: BackendFlags::any(),
            }
        }
    }

    #[repr(C)]
    pub enum SolverCreationError {
        NoSuchSolverFamily,
        NoBackendMatching,
    }

    impl<'alloc> Context<'alloc> {
        pub fn create_solver_sync<'a>(
            &'a self,
            _params: Parameters,
        ) -> Result<Box<crate::solver::ffi::Solver<'a>>, SolverCreationError> {
            use pollster::FutureExt as _;
            let solver =
                rtori_core::os_solver::Solver::create(rtori_core::os_solver::BackendFlags::CPU)
                    .block_on()
                    .unwrap();

            Ok(Box::new(crate::solver::ffi::Solver {
                ctx: self,
                inner: solver,
            }))
        }
    }
}
