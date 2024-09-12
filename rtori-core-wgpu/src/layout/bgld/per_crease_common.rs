pub const BIND_GROUP_PER_CREASE_GEOMETRY_NAME: &'static str = "rtori-bgl-per_crease_geometry";
pub const BIND_GROUP_PER_CREASE_GEOMETRY: wgpu::BindGroupLayoutDescriptor<'static> =
    wgpu::BindGroupLayoutDescriptor {
        label: Some(BIND_GROUP_PER_CREASE_GEOMETRY_NAME),
        entries: &[
            // crease_geometry
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Storage { read_only: true },
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
        ],
    };
