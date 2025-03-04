pub const BIND_GROUP_PER_CREASE_PHYSICS_NAME: &'static str = "rtori-bgl-per_crease_physics";
pub const BIND_GROUP_PER_CREASE_PHYSICS: wgpu::BindGroupLayoutDescriptor<'static> =
    wgpu::BindGroupLayoutDescriptor {
        label: Some(BIND_GROUP_PER_CREASE_PHYSICS_NAME),
        entries: &[
            // crease_physics (rw)
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Storage { read_only: false },
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
        ],
    };
