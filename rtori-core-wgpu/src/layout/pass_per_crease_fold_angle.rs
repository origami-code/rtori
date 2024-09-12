use std::borrow::Cow;

use super::pass::Pass;
pub struct PassPerCreaseFoldAngle(pub Pass);

impl PassPerCreaseFoldAngle {
    pub fn new(
        device: &wgpu::Device,
        bgl_positions_ro: &wgpu::BindGroupLayout,
        bgl_per_crease_common: &wgpu::BindGroupLayout,
    ) -> Self {
        let bgl_specific = device.create_bind_group_layout(
            &super::bgld::per_crease_fold_angles::BIND_GROUP_PER_CREASE_FOLD_ANGLES,
        );

        let shader_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("rtori-cs-per-crease-fold_angle"),
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!(
                "../../shaders/1-PerCrease-FoldAngles.wgsl"
            ))),
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("rtori-pl-per-crease-fold_angle"),
            bind_group_layouts: &[bgl_positions_ro, bgl_per_crease_common, &bgl_specific],
            push_constant_ranges: &[],
        });

        let compute_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("rtori-cp-per-crease-fold_angle"),
            layout: Some(&pipeline_layout),
            module: &shader_module,
            entry_point: "per_crease_fold_angles",
            compilation_options: Default::default(),
            cache: None,
        });

        Self(Pass {
            bind_group_layout: bgl_specific,
            compute_pipeline,
        })
    }
}
