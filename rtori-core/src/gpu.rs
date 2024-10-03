
use os_wgpu::wgpu;


#[derive(Debug)]
struct GPURunner {
    runner: os_wgpu::Runner,
    primary_queue: wgpu::Queue,
}

impl GPURunner {
    async fn create(backends: wgpu::Backends) -> Result<Self, ()> {
        // Create a GPU-based one
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: backends,
            dx12_shader_compiler: wgpu::Dx12Compiler::Dxc {
                dxil_path: Some(PathBuf::from("dxil.dll")),
                dxc_path: Some(PathBuf::from("dxcompiler.dll")),
            },
            ..Default::default()
        });

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptionsBase {
                power_preference: wgpu::PowerPreference::None,
                force_fallback_adapter: false,
                compatible_surface: None,
            })
            .await;

        if let Some(adapter) = adapter {
            let limits = os_wgpu::Runner::optimize_limits(adapter.limits(), None).unwrap();
            let features =
                os_wgpu::Runner::optimize_features(adapter.features(), None, &adapter.get_info());

            let res = adapter
                .request_device(
                    &wgpu::DeviceDescriptor {
                        label: Some("rtori-core-os-solver"),
                        required_features: features,
                        required_limits: limits,
                        memory_hints: Default::default(),
                    },
                    None,
                )
                .await;

            res.map(|(device, queue)| {
                let runner = rtori_core_wgpu::Runner::create(&device);
                Self {
                    runner,
                    primary_queue: queue,
                }
            })
            .map_err(|_e| ())
        } else {
            Err(())
        }
    }
}
