use std::borrow::Cow;
use super::pass::Pass;

pub struct PassPerNodeAccumulate(pub Pass);

impl PassPerNodeAccumulate {
    pub fn new(
        device: &wgpu::Device
    ) -> Self {
        let bgl_specific = device.create_bind_group_layout(&super::bgld::per_node_accumulate::BIND_GROUP_PER_NODE_ACCUMULATE);

        let shader_module =  device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("rtori-cs-per-node-accumulate"),
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!("../../../shaders/3-PerNode-Accumulate.wgsl")))
        });
        
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("rtori-pl-per-node-accumulate"),
            bind_group_layouts: &[
                &bgl_specific
            ],
            push_constant_ranges: &[]
        });

        let compute_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("rtori-cp-per-node-accumulate"),
            layout: Some(&pipeline_layout),
            module: &shader_module,
            entry_point: "per_node_accumulate",
            compilation_options: Default::default(),
            cache: None
        });

        Self(Pass{
            bind_group_layout: bgl_specific,
            compute_pipeline
        })
    }
}