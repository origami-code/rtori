use super::pass::Pass;
use std::borrow::Cow;

#[derive(Debug)]
pub struct PassPerNodeFace(pub Pass);

impl PassPerNodeFace {
    pub fn new(device: &wgpu::Device, bgl_positions_ro: &wgpu::BindGroupLayout) -> Self {
        let bgl_specific =
            device.create_bind_group_layout(&super::bgld::per_node_face::BIND_GROUP_PER_NODE_FACE);

        let shader_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("rtori-cs-per-node-face"),
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!(
                "../../shaders/2c0-PerNodeFace.wgsl"
            ))),
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("rtori-pl-per-node-face"),
            bind_group_layouts: &[&bgl_positions_ro, &bgl_specific],
            push_constant_ranges: &[],
        });

        let compute_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("rtori-cp-per-node-face"),
            layout: Some(&pipeline_layout),
            module: &shader_module,
            entry_point: "per_node_face",
            compilation_options: Default::default(),
            cache: None,
        });

        Self(Pass {
            bind_group_layout: bgl_specific,
            compute_pipeline,
        })
    }
}
