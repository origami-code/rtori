use std::path::PathBuf;

#[cfg(feature = "gpu")]
use rtori_core_wgpu as os_wgpu;
#[cfg(feature = "cpu")]
use rtori_core_simd as os_cpu;

use os_wgpu::wgpu;

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
struct GPURunner {
    runner: os_wgpu::Runner,
    primary_queue: wgpu::Queue
}

impl GPURunner {
    async fn create(
        backends: wgpu::Backends
    )  -> Result<Self, ()> {
        // Create a GPU-based one
        let instance = wgpu::Instance::new(
            wgpu::InstanceDescriptor{
                backends: backends,
                dx12_shader_compiler: wgpu::Dx12Compiler::Dxc { dxil_path: Some(PathBuf::from("dxil.dll")), dxc_path: Some(PathBuf::from("dxcompiler.dll")) },
                ..Default::default()
            }
        );

        let adapter = instance.request_adapter(&wgpu::RequestAdapterOptionsBase {
            power_preference: wgpu::PowerPreference::None,
            force_fallback_adapter: false,
            compatible_surface: None
        }).await;

        if let Some(adapter) = adapter {
            let limits = os_wgpu::Runner::optimize_limits(adapter.limits(), None).unwrap();
            let features = os_wgpu::Runner::optimize_features(adapter.features(), None, &adapter.get_info());

            let res = adapter.request_device(&wgpu::DeviceDescriptor {
                label: Some("rtori-core-os-solver"),
                required_features: features,
                required_limits: limits,
                memory_hints: Default::default()
            }, None).await;

            res.map(|(device, queue)| {
                let runner = rtori_core_wgpu::Runner::create(&device);
                Self {
                    runner,
                    primary_queue: queue
                }
            }).map_err(|_e| ())
        } else {
            Err(()) 
        }
    }
}

#[derive(Debug)]
pub enum Solver {
    #[cfg(feature = "cpu")]
    CPU(os_cpu::Simulator),
    #[cfg(feature = "gpu")]
    GPU(GPURunner)
}

impl Solver {
    pub async fn create(
        backends: BackendFlags
    ) -> Result<Self, ()> {
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
            
            GPURunner::create(wgpu_backends).await.map(|inner| Self::GPU(inner))
        } else if backends.intersects(BackendFlags::CPU) {
            // create a CPU-backed one
            unimplemented!()
        } else {
            // Invalid
            Err(())
        }
    }
}