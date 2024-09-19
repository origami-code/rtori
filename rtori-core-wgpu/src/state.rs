/*
 * Buffers:
 * - node_position_offset (double-buffered)
 * - crease_theta (double-buffered)
 * - node_position_original (gpu read-only)
 * - crease_nodes (gpu read-only, previously called creaseMeta2)
 * - face_vertex_indices (gpu read-only)
 * - face_normals (gpu-only intermediate buffer)
 * - crease_vectors (gpu read-only)
 * - crease_parameters (gpu read-only)
 * - crease_state (gpu-only intermediate [theta, thetaDiff])
 */

use std::{borrow::Cow, num::NonZeroU64};

use super::{
    bind_groups::BindGroups, layout::PipelineSetLayout, storage_buffers, uniform_buffers, ModelSize,
};

#[derive(Debug)]
pub struct State {
    params: ModelSize,
    bind_groups: BindGroups,
    backing_storage_buffer: storage_buffers::BackingStorageBuffer,
    backing_uniform_buffer: uniform_buffers::BackingUniformBuffer,
}

impl State {
    pub fn create(device: &wgpu::Device, params: ModelSize, layout: &PipelineSetLayout) -> Self {
        let limits = device.limits();

        let backing_storage_buffer = storage_buffers::BackingStorageBuffer::allocate(
            device,
            storage_buffers::Parameters {
                parameters: params,
                min_storage_alignment: limits.min_storage_buffer_offset_alignment,
            },
        );

        let backing_uniform_buffer = uniform_buffers::BackingUniformBuffer::allocate(
            device,
            limits.min_uniform_buffer_offset_alignment.into(),
        );

        let bind_groups = BindGroups::create(
            device,
            layout,
            &backing_storage_buffer.bindings().0,
            &backing_uniform_buffer.bindings(),
        );

        Self {
            params,
            bind_groups,
            backing_storage_buffer,
            backing_uniform_buffer,
        }
    }

    pub fn encode_pass(&self, layout: &PipelineSetLayout, pass: &mut wgpu::ComputePass) {
        pass.set_bind_group(0, &self.bind_groups.bg_positions_ro, &[]);

        {
            pass.set_pipeline(&layout.pass_per_face.0.compute_pipeline);
            pass.set_bind_group(1, &self.bind_groups.bg_per_face, &[]);
            pass.dispatch_workgroups(self.params.face_count.into(), 1, 1);

            pass.set_bind_group(1, &self.bind_groups.bg_per_crease_common, &[]);

            {
                pass.set_pipeline(&layout.pass_per_crease_fold_angle.0.compute_pipeline);
                pass.set_bind_group(2, &self.bind_groups.bg_per_crease_fold_angles, &[]);
                pass.dispatch_workgroups(self.params.crease_count.into(), 1, 1);

                pass.set_pipeline(&layout.pass_per_crease_physics.0.compute_pipeline);
                pass.set_bind_group(2, &self.bind_groups.bg_per_crease_physics, &[]);
                pass.dispatch_workgroups(self.params.crease_count.into(), 1, 1);

                pass.set_pipeline(&layout.pass_per_node_crease.0.compute_pipeline);
                pass.set_bind_group(2, &self.bind_groups.bg_per_node_crease, &[]);
                pass.dispatch_workgroups(self.params.node_crease_count.into(), 1, 1);
            }

            pass.set_pipeline(&layout.pass_per_node_beam.0.compute_pipeline);
            pass.set_bind_group(1, &self.bind_groups.bg_per_node_beam, &[]);
            pass.dispatch_workgroups(self.params.node_beam_count.into(), 1, 1);

            pass.set_pipeline(&layout.pass_per_node_face.0.compute_pipeline);
            pass.set_bind_group(1, &self.bind_groups.bg_per_node_face, &[]);
            pass.dispatch_workgroups(self.params.node_face_count.into(), 1, 1);
        }

        pass.set_pipeline(&layout.pass_per_node_accumulate.0.compute_pipeline);
        pass.set_bind_group(0, &self.bind_groups.bg_per_node_accumulate, &[]);
        pass.dispatch_workgroups(self.params.node_count.into(), 1, 1);
    }

    pub fn params(&self) -> &ModelSize {
        &self.params
    }

    pub fn clear(&mut self) {
        todo!()
    }
}
