pub const BIND_GROUP_PER_CREASE_FOLD_ANGLES_NAME: &'static str = "rtori-bgl-per_crease_fold_angles";
pub const BIND_GROUP_PER_CREASE_FOLD_ANGLES: wgpu::BindGroupLayoutDescriptor<'static> = wgpu::BindGroupLayoutDescriptor{
    label: Some(BIND_GROUP_PER_CREASE_FOLD_ANGLES_NAME),
    entries: &[
        // face_normals (ro)
        wgpu::BindGroupLayoutEntry {
            binding: 0,
            visibility: wgpu::ShaderStages::COMPUTE,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Storage { read_only: true },
                has_dynamic_offset: false,
                min_binding_size: None
            },
            count: None
        },
        // crease_fold_angles (rw)
        wgpu::BindGroupLayoutEntry {
            binding: 1,
            visibility: wgpu::ShaderStages::COMPUTE,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Storage { read_only: false },
                has_dynamic_offset: false,
                min_binding_size: None
            },
            count: None
        }
    ]
};

