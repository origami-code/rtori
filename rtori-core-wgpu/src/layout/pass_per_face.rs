use super::pass::Pass;
use std::borrow::Cow;

#[derive(Debug)]
pub struct PassPerFace(pub Pass);

impl PassPerFace {
    pub fn new(device: &wgpu::Device, common_bind_group_layout: &wgpu::BindGroupLayout) -> Self {
        let bgl_specific =
            device.create_bind_group_layout(&super::bgld::per_face::BIND_GROUP_PER_FACE);

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("rtori-pl-per-face"),
            bind_group_layouts: &[common_bind_group_layout, &bgl_specific],
            push_constant_ranges: &[],
        });

        let shader_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("rtori-cs-per-face"),
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!(
                "../../shaders/0-PerFace-CalculateNormal.wgsl"
            ))),
        });

        let compute_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("rtori-cp-per-face"),
            layout: Some(&pipeline_layout),
            module: &shader_module,
            entry_point: "per_face_calculate_normal",
            compilation_options: Default::default(),
            cache: None,
        });

        Self(Pass {
            bind_group_layout: bgl_specific,
            compute_pipeline,
        })
    }
}
