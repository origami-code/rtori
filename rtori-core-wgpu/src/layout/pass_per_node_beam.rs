use super::pass::Pass;
use std::borrow::Cow;

pub struct PassPerNodeBeam(pub Pass);

impl PassPerNodeBeam {
    pub fn new(device: &wgpu::Device, bgl_positions_ro: &wgpu::BindGroupLayout) -> Self {
        let bgl_specific =
            device.create_bind_group_layout(&super::bgld::per_node_beam::BIND_GROUP_PER_NODE_BEAM);

        let shader_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("rtori-cs-per-node-beam"),
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!(
                "../../shaders/2b0-PerNodeBeam.wgsl"
            ))),
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("rtori-pl-per-node-beam"),
            bind_group_layouts: &[&bgl_positions_ro, &bgl_specific],
            push_constant_ranges: &[],
        });

        let compute_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("rtori-cp-per-node-beam"),
            layout: Some(&pipeline_layout),
            module: &shader_module,
            entry_point: "per_node_beam",
            compilation_options: Default::default(),
            cache: None,
        });

        Self(Pass {
            bind_group_layout: bgl_specific,
            compute_pipeline,
        })
    }
}
