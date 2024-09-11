pub const BIND_GROUP_POSITIONS_RO_NAME: &'static str = "rtori-bgl-positions-ro";
pub const BIND_GROUP_POSITIONS_RO: wgpu::BindGroupLayoutDescriptor<'static> = wgpu::BindGroupLayoutDescriptor {
    label: Some(BIND_GROUP_POSITIONS_RO_NAME),
    entries: &[
        // unchanging
        wgpu::BindGroupLayoutEntry {
            binding: 0,
            visibility: wgpu::ShaderStages::COMPUTE,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Storage {
                    read_only: true
                },
                has_dynamic_offset: false,
                min_binding_size: None // could be set, but ?
            },
            count: None
        },
        // offset
        wgpu::BindGroupLayoutEntry {
            binding: 1,
            visibility: wgpu::ShaderStages::COMPUTE,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Storage {
                    read_only: true
                },
                has_dynamic_offset: false,
                min_binding_size: None // could be set, but ?
            },
            count: None
        },
    ]
};
