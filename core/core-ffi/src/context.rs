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
        pub(crate) context: rtori_core::Context<crate::A<'alloc>>,
        _marker: core::marker::PhantomData<&'alloc crate::A<'alloc>>,
    }

    impl<'a> Context<'a> {
        #[diplomat::attr(auto, constructor)]
        pub fn global() -> Box<Self, crate::A<'a>> {
            Box::new_in(
                Self {
                    allocator: alloc::alloc::Global,
                    context: rtori_core::Context::new(alloc::alloc::Global),
                    _marker: core::marker::PhantomData,
                },
                alloc::alloc::Global,
            )
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

    #[derive(Clone, Copy, Default)]
    #[repr(C)]
    pub struct BackendFlags {
        pub value: u8,
    }

    impl BackendFlags {
        pub const CPU: Self = Self {
            value: rtori_core::BackendFlags::CPU.bits(),
        };
        pub const CPU_MT: Self = Self {
            value: rtori_core::BackendFlags::CPU_MT.bits(),
        };

        pub const GPU_METAL: Self = Self {
            value: rtori_core::BackendFlags::GPU_METAL.bits(),
        };
        pub const GPU_VULKAN: Self = Self {
            value: rtori_core::BackendFlags::GPU_VULKAN.bits(),
        };
        pub const GPU_DX12: Self = Self {
            value: rtori_core::BackendFlags::GPU_DX12.bits(),
        };
        pub const GPU_WEBGPU: Self = Self {
            value: rtori_core::BackendFlags::GPU_WEBGPU.bits(),
        };

        pub fn gpu_metal() -> Self {
            Self::GPU_METAL
        }

        pub fn gpu_vulkan() -> Self {
            Self::GPU_VULKAN
        }

        pub fn gpu_dx12() -> Self {
            Self::GPU_DX12
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
            Self::CPU
        }

        pub fn cpu_mt() -> Self {
            Self::CPU_MT
        }

        pub fn cpu_any() -> Self {
            Self::CPU
        }

        #[diplomat::attr(auto, constructor)]
        pub fn any() -> Self {
            Self {
                value: Self::cpu_any().value | Self::gpu_any().value,
            }
        }

        pub fn or(self, other: BackendFlags) -> BackendFlags {
            Self {
                value: self.value | other.value,
            }
        }

        pub fn has(self, subset: BackendFlags) -> bool {
            (self.value | subset.value) == self.value
        }
    }

    #[repr(C)]
    pub struct Parameters {
        pub family: crate::solver::ffi::SolverFamily,
        /// Acceptable backends
        pub backend: BackendFlags,
    }

    impl Parameters {
        #[diplomat::attr(auto, constructor)]
        pub fn new() -> Self {
            Self {
                family: crate::solver::ffi::SolverFamily::OrigamiSimulator,
                backend: BackendFlags::any(),
            }
        }
    }

    #[derive(Debug)]
    #[repr(C)]
    pub enum SolverCreationError {
        NoSuchSolverFamily,
        NoBackendMatching,
    }

    impl SolverCreationError {
        #[diplomat::attr(auto, stringifier)]
        pub fn format(&self, out: &mut DiplomatWrite) {
            use std::fmt::Write;
            match self {
                Self::NoSuchSolverFamily => write!(out, "no such solver family"),
                Self::NoBackendMatching => write!(
                    out,
                    "no backend matching the given flags for the given family"
                ),
            }
            .unwrap();
        }
    }

    impl<'alloc> Context<'alloc> {
        pub fn create_solver_sync<'a>(
            &'a self,
            params: Parameters,
        ) -> Result<Box<crate::solver::ffi::Solver<'a>>, SolverCreationError> {
            let family = match params.family {
                crate::solver::ffi::SolverFamily::OrigamiSimulator => {
                    rtori_core::SolverFamily::OrigamiSimulator
                }
            };
            let backends = rtori_core::BackendFlags::from_bits_truncate(params.backend.value);

            use pollster::FutureExt as _;
            let solver = rtori_core::Solver::create(&self.context, family, backends)
                .block_on()
                .unwrap();

            Ok(Box::new(crate::solver::ffi::Solver {
                ctx: self,
                inner: solver,
            }))
        }
    }
}
