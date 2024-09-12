pub struct StorageBindings<'backing> {
    /// cpy-src
    pub node_positions_offset: wgpu::BufferBinding<'backing>,
    /// cpy-dst
    pub node_positions_unchanging: wgpu::BufferBinding<'backing>,
    /// gpu-only
    pub node_velocity: wgpu::BufferBinding<'backing>,
    /// cpy-src
    pub node_error: wgpu::BufferBinding<'backing>,
    /// cpy-dst
    pub node_external_forces: wgpu::BufferBinding<'backing>,
    /// cpy-dst
    pub node_configs: wgpu::BufferBinding<'backing>,
    /// cpy-dst
    pub node_geometry: wgpu::BufferBinding<'backing>,

    /// cpy-dst
    pub crease_geometry: wgpu::BufferBinding<'backing>,
    /// gpu-only
    pub crease_fold_angles: wgpu::BufferBinding<'backing>,
    /// gpu-only
    pub crease_physics: wgpu::BufferBinding<'backing>,
    /// cpy-dst
    pub crease_parameters: wgpu::BufferBinding<'backing>,

    /// cpy-dst
    pub face_indices: wgpu::BufferBinding<'backing>,
    /// cpy-dst
    pub face_nominal_angles: wgpu::BufferBinding<'backing>,
    /// gpu-only
    pub face_normals: wgpu::BufferBinding<'backing>,

    /// cpy-dst
    pub node_creases: wgpu::BufferBinding<'backing>,
    /// gpu-only
    pub node_creases_constraint_forces: wgpu::BufferBinding<'backing>,

    /// cpy-dst
    pub node_beams: wgpu::BufferBinding<'backing>,
    /// gpu-only
    pub node_beams_constraint_forces: wgpu::BufferBinding<'backing>,

    /// cpy-dst
    pub node_faces: wgpu::BufferBinding<'backing>,
    /// gpu-only
    pub node_faces_constraint_forces: wgpu::BufferBinding<'backing>,
}

impl StorageBindings<'_> {
    pub const COUNT: usize = 20;
}
