use super::{layout::PipelineSetLayout, storage_buffers, uniform_buffers};

pub struct BindGroups {
    pub bg_positions_ro: wgpu::BindGroup,
    pub bg_per_face: wgpu::BindGroup,
    pub bg_per_crease_common: wgpu::BindGroup,
    pub bg_per_crease_fold_angles: wgpu::BindGroup,
    pub bg_per_crease_physics: wgpu::BindGroup,
    pub bg_per_node_crease: wgpu::BindGroup,
    pub bg_per_node_beam: wgpu::BindGroup,
    pub bg_per_node_face: wgpu::BindGroup,
    pub bg_per_node_accumulate: wgpu::BindGroup,
}

impl BindGroups {
    pub fn create(
        device: &wgpu::Device,
        layout: &PipelineSetLayout,
        storage_buffers: &storage_buffers::StorageBindings,
        uniform_buffers: &uniform_buffers::UniformBindings,
    ) -> Self {
        let bg_positions_ro = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("rtori-bg_positions_ro"),
            layout: &layout.bgl_positions_ro,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Buffer(
                        storage_buffers.node_positions_unchanging.clone(),
                    ),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Buffer(
                        storage_buffers.node_positions_offset.clone(),
                    ),
                },
            ],
        });

        let bg_per_face = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("rtori-bg_per_face"),
            layout: &layout.pass_per_face.0.bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Buffer(storage_buffers.face_indices.clone()),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Buffer(storage_buffers.face_normals.clone()),
                },
            ],
        });

        let bg_per_crease_common = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("rtori-bg_per_crease_common"),
            layout: &layout.bgl_per_crease_common,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::Buffer(storage_buffers.crease_geometry.clone()),
            }],
        });

        let bg_per_crease_fold_angles = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("rtori-bg_per_crease_fold_angles"),
            layout: &layout.pass_per_crease_fold_angle.0.bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Buffer(storage_buffers.face_normals.clone()),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Buffer(
                        storage_buffers.crease_fold_angles.clone(),
                    ),
                },
            ],
        });

        let bg_per_crease_physics = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("rtori-bg_per_crease_physics"),
            layout: &layout.pass_per_crease_physics.0.bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::Buffer(storage_buffers.crease_physics.clone()),
            }],
        });

        let bg_per_node_crease = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("rtori-bg_per_node_crease"),
            layout: &layout.pass_per_node_crease.0.bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Buffer(
                        storage_buffers.node_creases_constraint_forces.clone(),
                    ),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Buffer(storage_buffers.node_creases.clone()),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::Buffer(
                        storage_buffers.crease_fold_angles.clone(),
                    ),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: wgpu::BindingResource::Buffer(storage_buffers.crease_physics.clone()),
                },
                wgpu::BindGroupEntry {
                    binding: 4,
                    resource: wgpu::BindingResource::Buffer(
                        storage_buffers.crease_parameters.clone(),
                    ),
                },
                wgpu::BindGroupEntry {
                    binding: 5,
                    resource: wgpu::BindingResource::Buffer(
                        uniform_buffers.crease_percentage.clone(),
                    ),
                },
                wgpu::BindGroupEntry {
                    binding: 6,
                    resource: wgpu::BindingResource::Buffer(storage_buffers.face_indices.clone()),
                },
                wgpu::BindGroupEntry {
                    binding: 7,
                    resource: wgpu::BindingResource::Buffer(storage_buffers.face_normals.clone()),
                },
            ],
        });

        let bg_per_node_beam = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("rtori-bg_per_node_beam"),
            layout: &layout.pass_per_node_beam.0.bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Buffer(
                        storage_buffers.node_beams_constraint_forces.clone(),
                    ),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Buffer(storage_buffers.node_velocity.clone()),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::Buffer(storage_buffers.node_beams.clone()),
                },
            ],
        });

        let bg_per_node_face = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("rtori-bg_per_node_face"),
            layout: &layout.pass_per_node_face.0.bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Buffer(
                        storage_buffers.node_faces_constraint_forces.clone(),
                    ),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Buffer(storage_buffers.node_velocity.clone()),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::Buffer(storage_buffers.node_faces.clone()),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: wgpu::BindingResource::Buffer(storage_buffers.face_indices.clone()),
                },
                wgpu::BindGroupEntry {
                    binding: 4,
                    resource: wgpu::BindingResource::Buffer(storage_buffers.face_normals.clone()),
                },
                wgpu::BindGroupEntry {
                    binding: 5,
                    resource: wgpu::BindingResource::Buffer(
                        storage_buffers.face_nominal_angles.clone(),
                    ),
                },
            ],
        });

        let bg_per_node_accumulate = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("rtori-bg_per_node_accumulate"),
            layout: &layout.pass_per_node_accumulate.0.bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Buffer(
                        storage_buffers.node_positions_offset.clone(),
                    ),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Buffer(storage_buffers.node_velocity.clone()),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::Buffer(storage_buffers.node_error.clone()),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: wgpu::BindingResource::Buffer(
                        storage_buffers.node_creases_constraint_forces.clone(),
                    ),
                },
                wgpu::BindGroupEntry {
                    binding: 4,
                    resource: wgpu::BindingResource::Buffer(
                        storage_buffers.node_beams_constraint_forces.clone(),
                    ),
                },
                wgpu::BindGroupEntry {
                    binding: 5,
                    resource: wgpu::BindingResource::Buffer(
                        storage_buffers.node_faces_constraint_forces.clone(),
                    ),
                },
                wgpu::BindGroupEntry {
                    binding: 6,
                    resource: wgpu::BindingResource::Buffer(storage_buffers.node_geometry.clone()),
                },
                wgpu::BindGroupEntry {
                    binding: 7,
                    resource: wgpu::BindingResource::Buffer(storage_buffers.node_configs.clone()),
                },
                wgpu::BindGroupEntry {
                    binding: 8,
                    resource: wgpu::BindingResource::Buffer(
                        storage_buffers.node_external_forces.clone(),
                    ),
                },
                wgpu::BindGroupEntry {
                    binding: 9,
                    resource: wgpu::BindingResource::Buffer(uniform_buffers.dt.clone()),
                },
            ],
        });

        Self {
            bg_positions_ro,
            bg_per_face,
            bg_per_crease_common,
            bg_per_crease_fold_angles,
            bg_per_crease_physics,
            bg_per_node_crease,
            bg_per_node_beam,
            bg_per_node_face,
            bg_per_node_accumulate,
        }
    }
}
