use std::num::NonZeroU64;

use wgpu::BufferViewMut;

use crate::extractor::ExtractorRanges;
use crate::loader::{LoadRanges, Loader, LoaderMappedTarget, LoaderRange, LoaderStagingBelt};

use super::parameters::Parameters;
use super::storage_bindings::StorageBindings;
use crate::state::ExtractFlags;

#[derive(Debug)]
pub struct ReusableBuffer {
    pub buffer: wgpu::Buffer,
    pub mapped: bool
}

impl ReusableBuffer {
    pub fn destroy(self) {
        self.buffer.destroy();
        drop(self)
    }
}

#[derive(Debug)]
pub struct BackingStorageBuffer {
    pub buffer: wgpu::Buffer,
    pub parameters: Parameters,
    mapped: bool
}

impl BackingStorageBuffer {
    const BUFFER_USAGE: wgpu::BufferUsages = wgpu::BufferUsages::from_bits_truncate(
        wgpu::BufferUsages::COPY_SRC.bits()
            | wgpu::BufferUsages::COPY_DST.bits()
            | wgpu::BufferUsages::STORAGE.bits());

    const fn craft_buffer_descriptor(size: NonZeroU64) -> wgpu::BufferDescriptor<'static> {
        wgpu::BufferDescriptor {
            label: Some("rtori-backing-storage-buffer"),
            size: size.get(),
            usage: Self::BUFFER_USAGE,
            mapped_at_creation: true,
        }
    }

    pub fn would_reallocate(&self, parameters: Parameters) -> bool {
         let previous_min_size = self.buffer.size();
         let new_min_size = parameters.min_size();
         new_min_size.get() > previous_min_size
    }

    /// returns the previous buffer if it reallocated
    pub fn reallocate_if_needed(&mut self, device: &wgpu::Device, parameters: Parameters) -> Option<ReusableBuffer> {
        let previous_min_size = self.buffer.size();
        let new_min_size = parameters.min_size();
        if new_min_size.get() <= previous_min_size {
            return None;
        }

        let new_buffer = device.create_buffer(&Self::craft_buffer_descriptor(new_min_size));
        let previous = ReusableBuffer {
            buffer: std::mem::replace(&mut self.buffer, new_buffer),
            mapped: std::mem::replace(&mut self.mapped, true)
        };

        Some(previous)
    }

    pub fn allocate(device: &wgpu::Device, parameters: Parameters) -> Self {
        let min_size = parameters.min_size();
        let buffer = device.create_buffer(&Self::craft_buffer_descriptor(min_size));

        Self { buffer, parameters, mapped: true }
    }

    pub fn bindings(&self) -> (StorageBindings, NonZeroU64) {
        let (mut g, total_size) = {
            let (bindings_ranges, total_size) = self.parameters.binding_ranges();

            let g = {
                let mut idx = 0;
                let g = move || {
                    let (offset, size) = bindings_ranges[idx];
                    let binding = wgpu::BufferBinding{
                        buffer: &self.buffer,
                        offset,
                        size: Some(size)
                    };
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

    fn craft_load_ranges<'cur>(&'cur self) -> LoadRanges
    {
        let ranges = self.parameters.binding_ranges().0;
        let range_for_order = |order: usize| {
            let range = ranges[order];
            LoaderRange {
                offset: range.0,
                size: range.1
            }
        };

        LoadRanges {
            node_positions_unchanging: range_for_order(Parameters::ORDER_NODE_POSITIONS_UNCHANGING),
            node_external_forces: range_for_order(Parameters::ORDER_NODE_EXTERNAL_FORCES),
            node_configs: range_for_order(Parameters::ORDER_NODE_CONFIGS),
            node_geometry: range_for_order(Parameters::ORDER_NODE_GEOMETRY),
            crease_geometry: range_for_order(Parameters::ORDER_CREASE_GEOMETRY),
            crease_parameters: range_for_order(Parameters::ORDER_CREASE_PARAMETERS),
            face_indices: range_for_order(Parameters::ORDER_FACE_INDICES),
            face_nominal_angles: range_for_order(Parameters::ORDER_FACE_NOMINAL_TRIANGLES),
            node_creases: range_for_order(Parameters::ORDER_NODE_CREASES),
            node_beams: range_for_order(Parameters::ORDER_NODE_BEAMS),
            node_faces: range_for_order(Parameters::ORDER_NODE_FACES)
        }
    }

    pub fn load_mapped<'a>(&'a self) -> LoaderMappedTarget<'a> {
        LoaderMappedTarget::new(&self.buffer, self.craft_load_ranges())
    }

    pub fn load_staging<'a>(
        &'a mut self,
        encoder: &'a mut wgpu::CommandEncoder,
        staging_belt: &'a mut wgpu::util::StagingBelt,
        device: &'a wgpu::Device
    ) -> LoaderStagingBelt<'a> {
       LoaderStagingBelt::new(
        &self.buffer,
        encoder,
        staging_belt,
        device,
        self.craft_load_ranges()
       )
    }

    /// If the buffer is mapped, then it returns a Loader::Mapped, otherwise it uses the supplied staging belt and returns a Loader::StagingBelt
    pub fn load_optimal<'a>(&'a mut self, staging: (&'a mut wgpu::CommandEncoder, &'a mut wgpu::util::StagingBelt, &'a wgpu::Device)) -> Loader<'a> {
        if self.mapped {
            Loader::Mapped(
                self.load_mapped()
            )
        } else {
            Loader::StagingBelt(
                self.load_staging(staging.0, staging.1, staging.2)
            )
        }
    }

    pub fn extract_map_size(
        &self,
        kind: ExtractFlags
    ) -> u64 {
        let mut acc = 0;
        if kind.contains(ExtractFlags::NodePositions) {
            acc += todo!():
        }
        if kind.contains(ExtractFlags::NodePositions) {
            acc += todo!()
        }
        acc
        
    }

    pub fn extract_map<'a>(
        &'a self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        mappable_buffer: &wgpu::Buffer,
        kind: ExtractFlags,
        callback: impl FnOnce(Result<crate::extractor::ExtractorMappedTarget<'_>, wgpu::BufferAsyncError>) + wgpu::WasmNotSend + 'static
    )-> Result<bool, ()> {
        if kind.is_empty() {
            return Ok(false);
        }

        let required_size = self.extract_map_size(kind);

        if mappable_buffer.size() < required_size {
            return Err(());
        }

        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("rtori-extract-encoder")
        });

        let bindings = self.parameters.binding_ranges().0;
        
        let mut dst_offset = 0;

        let node_positions_offset_range = if kind.contains(ExtractFlags::NodePositions) {
            let (offset, size) = bindings[Parameters::ORDER_NODE_POSITIONS_OFFSETS];

            encoder.copy_buffer_to_buffer(
                &self.buffer,
                offset,
                &mappable_buffer,
                dst_offset,
                size.get()
            );

            Some((dst_offset + offset, size))
        } else {
            None
        };
        dst_offset += node_positions_offset_range.map(|r| r.1.get()).unwrap_or(0);
        
        let node_error_range = if kind.contains(ExtractFlags::NodePositions) {
            let (offset, size) = bindings[Parameters::ORDER_NODE_ERROR];

            encoder.copy_buffer_to_buffer(
                &self.buffer,
                offset,
                &mappable_buffer,
                dst_offset,
                size.get()
            );
            dst_offset + size.get();

            Some(((dst_offset + offset), size))
        } else {
            None
        };
        dst_offset += node_error_range.map(|r| r.1.get()).unwrap_or(0);

        let command_buffer = encoder.finish();
        queue.submit(Some(command_buffer));
        
        mappable_buffer
            .slice(..required_size)
            .map_async(wgpu::MapMode::Read, move |result| {
                if let Err(e) = result {
                    callback(Err(e));
                    return;
                }

                let mapped = crate::extractor::ExtractorMappedTarget::new(
                    mappable_buffer,
                    crate::extractor::ExtractorRanges {
                        node_position_offset: node_positions_offset_range.map(|(offset, size)| LoaderRange{
                            offset,
                            size
                        }),
                        node_error: node_error_range.map(|(offset, size)| LoaderRange {
                            offset,
                            size
                        })
                    }
                );

                callback(Ok(mapped))
            });
        
        Ok(true)
    }

    pub fn unmap(&mut self) {
        self.mapped = false;
        self.buffer.unmap();
    }
}
