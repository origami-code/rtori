pub struct Pass {
    pub bind_group_layout: wgpu::BindGroupLayout,
    pub compute_pipeline: wgpu::ComputePipeline
}

impl Pass {
    pub fn new(
        bind_group_layout: wgpu::BindGroupLayout,
        compute_pipeline: wgpu::ComputePipeline
    ) -> Self {
        Self {
            bind_group_layout,
            compute_pipeline
        }
    }
}
