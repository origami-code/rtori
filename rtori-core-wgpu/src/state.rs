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

use std::{borrow::Cow, fmt::Debug, num::NonZeroU64};

use wgpu::BufferAsyncError;

use crate::{extractor::ExtractorMappedTarget, loader, storage_buffers::Parameters};

use super::{
    bind_groups::BindGroups, layout::PipelineSetLayout, storage_buffers, uniform_buffers, ModelSize,
};

bitflags::bitflags!{
    #[derive(Debug, Copy, Clone, PartialEq)]
    pub struct ExtractFlags: u8 {
        const NodePositions = 1 << 0;
        const NodeError = 1 << 1;
        const Everything = (1 << 0) | (1 << 1);
    }
}

#[derive(Debug)]
pub struct State<'state> {
    device: &'state wgpu::Device,
    params: ModelSize,
    bind_groups: BindGroups,
    backing_storage_buffer: storage_buffers::BackingStorageBuffer,
    backing_uniform_buffer: uniform_buffers::BackingUniformBuffer,
    
    /// for tuning parameters (small chunks)
    tuning_belt: wgpu::util::StagingBelt,

    /// This is a cpu-resident buffer
    download_buffer: Option<wgpu::Buffer>,

    loaded: bool
}


impl<'state> State<'state> {
    /// returns true if there is a need to refill the data now
    /// even if we didn't reallocate, if the parameters change, the buffer bindings are incorrect and thus garbage data's flying around
    pub fn recreate(
        &mut self,
        params: ModelSize,
        layout: &PipelineSetLayout) -> bool {
        if self.params == params {
            return false;
        }

        let limits = self.device.limits();

        // Uniforms don't change (for now)
        let changed = self.backing_storage_buffer.reallocate_if_needed(
            self.device,
            storage_buffers::Parameters {
                parameters: params,
                min_storage_alignment: limits.min_storage_buffer_offset_alignment,
            },
        );
        self.loaded = false;

        if let Some(buf) = changed {
            // TODO: reuse buffers ?
            buf.destroy();

            // Recreate the bind groups
            self.bind_groups = BindGroups::create(
                self.device,
                layout,
                &self.backing_storage_buffer.bindings().0,
                &self.backing_uniform_buffer.bindings(),
            );
        }

        true
    }

    pub fn create(device: &'state wgpu::Device, params: ModelSize, layout: &PipelineSetLayout) -> Self {
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

        let tuning_belt = wgpu::util::StagingBelt::new(64); // Fix with proper size

        let download_buffer = None;

        Self {
            device,
            params,
            bind_groups,
            backing_storage_buffer,
            backing_uniform_buffer,
            tuning_belt,
            download_buffer,
            loaded: false
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

    pub fn load(&mut self) -> Result<StateLoader<'_>, ()> {
        if self.loaded {
            return Err(());
        }

        Ok(StateLoader {
            loader: self.backing_storage_buffer.load_mapped()
        })
    }

    pub fn mark_loaded(&mut self) {
        if self.loaded {
            panic!("got marked loaded even though we already were");
        }

        // Unmap
        self.backing_storage_buffer.unmap();
        self.loaded = true;
    }

    pub fn extract(
        &mut self,
        queue: &wgpu::Queue,
        kind: ExtractFlags,
        callback: impl FnOnce(Result<ExtractorMappedTarget<'_>, BufferAsyncError>) + wgpu::WasmNotSend + 'static
    ) -> Result<bool, ()>
    {
        if !self.loaded {
            // cannot extract a non-loaded thing
            return Err(());
        }

        let required_size = self.backing_storage_buffer.extract_map_size(kind);

        let buffer = self.download_buffer.take().unwrap_or_else(|| self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("rtori-extract-buffer"),
            size: required_size,
            usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false
        }));

        self.backing_storage_buffer.extract_map(
            &self.device,
            &queue,
            &buffer,
            kind,
            callback
        )
    }
}

#[repr(transparent)]
pub struct StateLoader<'a> {
    pub loader: loader::LoaderMappedTarget<'a>
}

impl<'a> StateLoader<'a> {
    pub fn finish(self, state: &mut State) {
        state.mark_loaded();
    }
}
