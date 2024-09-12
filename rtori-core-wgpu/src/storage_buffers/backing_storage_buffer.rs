use std::num::NonZeroU64;

use super::parameters::Parameters;
use super::storage_bindings::StorageBindings;

pub struct BackingStorageBuffer {
    pub buffer: wgpu::Buffer,
    pub parameters: Parameters,
}

impl BackingStorageBuffer {
    pub fn allocate(device: &wgpu::Device, parameters: Parameters) -> Self {
        let min_size = parameters.min_size();
        let buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("rtori-backing-storage-buffer"),
            size: min_size,
            usage: wgpu::BufferUsages::COPY_SRC
                | wgpu::BufferUsages::COPY_DST
                | wgpu::BufferUsages::STORAGE,
            mapped_at_creation: true,
        });

        Self { buffer, parameters }
    }

    pub fn bindings(&self) -> (StorageBindings, u64) {
        let (mut g, total_size) = {
            let (mut bindings, total_size) = {
                let mut bindings: [Option<wgpu::BufferBinding<'_>>; StorageBindings::COUNT] =
                    [const { None }; StorageBindings::COUNT];
                let mut idx = 0;
                let f = |offset: u64, size: u64| {
                    let binding = wgpu::BufferBinding {
                        buffer: &self.buffer,
                        offset: offset,
                        size: Some(NonZeroU64::try_from(size).unwrap()),
                    };
                    (&mut bindings)[idx] = Some(binding);
                    *(&mut idx) += 1;
                };

                let total_size = self.parameters.apply_for_each_binding(f);

                (bindings, total_size)
            };

            let g = {
                let mut idx = 0;
                let g = move || {
                    let binding = bindings[idx].take().expect("Should always be in bounds");
                    *(&mut idx) += 1;
                    return binding;
                };
                g
            };
            (g, total_size)
        };

        (
            StorageBindings {
                node_positions_offset: g(),
                node_positions_unchanging: g(),
                node_velocity: g(),
                node_error: g(),
                node_external_forces: g(),
                node_configs: g(),
                node_geometry: g(),
                crease_geometry: g(),
                crease_fold_angles: g(),
                crease_physics: g(),
                crease_parameters: g(),
                face_indices: g(),
                face_nominal_angles: g(),
                face_normals: g(),
                node_creases: g(),
                node_creases_constraint_forces: g(),
                node_beams: g(),
                node_beams_constraint_forces: g(),
                node_faces: g(),
                node_faces_constraint_forces: g(),
            },
            total_size,
        )
    }
}
