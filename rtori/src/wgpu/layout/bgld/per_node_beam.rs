pub const BIND_GROUP_PER_NODE_BEAM_NAME: &'static str = "rtori-bgl-per_node_beam";
pub const BIND_GROUP_PER_NODE_BEAM: wgpu::BindGroupLayoutDescriptor<'static> = wgpu::BindGroupLayoutDescriptor{
    label: Some(BIND_GROUP_PER_NODE_BEAM_NAME),
    entries: &[
        // node_beam_constraint_forces (rw)
        wgpu::BindGroupLayoutEntry {
            binding: 0,
            visibility: wgpu::ShaderStages::COMPUTE,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Storage { read_only: false },
                has_dynamic_offset: false,
                min_binding_size: None
            },
            count: None
        },
        // node_velocity (ro)
        wgpu::BindGroupLayoutEntry {
            binding: 1,
            visibility: wgpu::ShaderStages::COMPUTE,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Storage { read_only: true },
                has_dynamic_offset: false,
                min_binding_size: None
            },
            count: None
        },
        // node_beams (ro)
        wgpu::BindGroupLayoutEntry {
            binding: 2,
            visibility: wgpu::ShaderStages::COMPUTE,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Storage { read_only: true },
                has_dynamic_offset: false,
                min_binding_size: None
            },
            count: None
        }
    ]
};

