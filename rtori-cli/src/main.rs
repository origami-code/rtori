use std::{thread, time::Duration};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let instance = wgpu::Instance::default();

    let adapter = instance
        .request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::default(),
            compatible_surface: None,
            force_fallback_adapter: false
        })
        .await
        .unwrap();

    let minimum_limits = {
        let mut default = wgpu::Limits::default();
        default.max_storage_buffers_per_shader_stage = 9;
        default
    };

    let (device, queue) = adapter
        .request_device(&wgpu::DeviceDescriptor{
            required_limits: minimum_limits,
            ..Default::default()
        }, None)
        .await
        .unwrap();

    let mut runner = rtori_core_wgpu::Runner::create(&device);
    runner.prepare(&device, &rtori_core_wgpu::ModelSize { node_count: 6, crease_count: 12, face_count: 10, node_beam_count: 16, node_crease_count: 14, node_face_count: 19 });
    

    thread::sleep(Duration::from_secs(15));
    Ok(())
}
