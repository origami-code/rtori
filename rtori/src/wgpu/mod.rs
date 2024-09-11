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

use wgpu::{util::DeviceExt, ShaderStages};

mod layout;
use layout::PipelineSetLayout;

struct PipelineSet {
    params: Parameters,
    layout: PipelineSetLayout,
    
    bg_positions_ro: wgpu::BindGroup,
    bg_per_face: wgpu::BindGroup,
    bg_per_crease_common: wgpu::BindGroup,
    bg_per_crease_fold_angles: wgpu::BindGroup,
    bg_per_crease_physics: wgpu::BindGroup,
    bg_per_node_crease: wgpu::BindGroup,
    bg_per_node_beam: wgpu::BindGroup,
    bg_per_node_face: wgpu::BindGroup,
    bg_per_node_accumulate: wgpu::BindGroup,
}

impl PipelineSet {
    pub fn encode_pass(&self, pass: &mut wgpu::ComputePass) {
        pass.set_bind_group(
            0,
            &self.bg_positions_ro,
            &[]
        );

        {
            pass.set_pipeline(&self.layout.pass_per_face.0.compute_pipeline);
            pass.set_bind_group(
                1,
                &self.bg_per_face,
                &[]
            );
            pass.dispatch_workgroups(self.params.face_count.into(), 1, 1);
            
            pass.set_bind_group(
                1,
                &self.bg_per_crease_common,
                &[]
            );

            {
                pass.set_pipeline(&self.layout.pass_per_crease_fold_angle.0.compute_pipeline);
                pass.set_bind_group(
                    2,
                    &self.bg_per_crease_fold_angles,
                    &[]
                );
                pass.dispatch_workgroups(self.params.crease_count.into(), 1, 1);

                pass.set_pipeline(&self.layout.pass_per_crease_physics.0.compute_pipeline);
                pass.set_bind_group(
                    2,
                    &self.bg_per_crease_physics,
                    &[]
                );
                pass.dispatch_workgroups(self.params.crease_count.into(), 1, 1);

                pass.set_pipeline(&self.layout.pass_per_node_crease.0.compute_pipeline);
                pass.set_bind_group(
                    2,
                    &self.bg_per_node_crease,
                    &[]
                );
                pass.dispatch_workgroups(self.params.node_crease_count.into(), 1, 1);

            }

            pass.set_pipeline(&self.layout.pass_per_node_beam.0.compute_pipeline);
            pass.set_bind_group(
                1,
                &self.bg_per_node_beam,
                &[]
            );
            pass.dispatch_workgroups(self.params.node_beam_count.into(), 1, 1);

            pass.set_pipeline(&self.layout.pass_per_node_face.0.compute_pipeline);
            pass.set_bind_group(
                1,
                &self.bg_per_node_face,
                &[]
            );
            pass.dispatch_workgroups(self.params.node_face_count.into(), 1, 1);
        }

        pass.set_pipeline(&self.layout.pass_per_node_accumulate.0.compute_pipeline);
        pass.set_bind_group(
            0,
            &self.bg_per_node_accumulate,
            &[]
        );
        pass.dispatch_workgroups(self.params.node_count.into(), 1, 1);
    }
}

struct GPURunner {

}

struct Parameters {
    node_count: u16,
    crease_count: u16,
    face_count: u16,
    node_beam_count: u16,
    node_crease_count: u16,
    node_face_count: u16
}

impl GPURunner {
    pub fn create(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        parameters: &Parameters
    ) {
        // TODO: initialize from x, y, z components
        let node_positions_unchanging = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("rtori-buf-node_positions_unchanging"),
            contents: &vec![0; usize::from(parameters.node_count) * 4 /* sizeof float */ * 3 /* x, y, z */ ],
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::STORAGE,
        });

        let node_positions_offsets_a = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("rtori-buf-node_positions_offsets_a"),
            contents: &vec![0; usize::from(parameters.node_count) * 4 /* sizeof float */ * 3 /* x, y, z */ ],
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::COPY_SRC | wgpu::BufferUsages::STORAGE
        });

        let node_positions_offsets_b = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("rtori-buf-node_positions_offsets_b"),
            size: usize::from(parameters.node_count) * 4 /* sizeof float */ * 3 /* x, y, z */ ,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::COPY_SRC | wgpu::BufferUsages::STORAGE,
            mapped_at_creation: false
        });

        let velocity = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("rtori-buf-node_positions_offsets_b"),
            size: parameters.node_count.into() * 4 /* sizeof float */ * 3 /* x, y, z */ ,
            usage: wgpu::BufferUsages::STORAGE,
            mapped_at_creation: false
        });

        let error = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("rtori-buf-node_error"),
            size: parameters.node_count.into() * 4 /* f32 */ * 1,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
            mapped_at_creation: false
        });

        let node_external_forces = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("rtori-buf-node_external_forces"),
            size: parameters.node_count.into() * 4 * 3,
            usage: wgpu::BufferUsages::STORAGE,
            mapped_at_creation: true
        });

        let face_node_indices = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("rtori-buf-face_node_indices"),
            size: parameters.face_count.into() * 2 /* sizeof u16 */ * 3 /* number of nodes per triangle */,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::STORAGE,
            mapped_at_creation: false
        });

        let face_normals = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("rtori-buf-face_normals"),
            size: parameters.face_count.into() * 4 /* sizeof f32 */ * 3 /* number of  components (x, y, z) */,
            usage: wgpu::BufferUsages::STORAGE,
            mapped_at_creation: false
        });

        let crease_geometry = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("rtori-buf-crease_geometry"),
            size: parameters.crease_count.into() * 2 /* sizeof u16 */ * (2 /* number of complement nodes */ + 2 /* adjacent nodes index */ + 2 /* face indices == normal indices */),
            usage: wgpu::BufferUsages::STORAGE,
            mapped_at_creation: true
        });
        
        // k & d
        let crease_parameters = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("rtori-buf-crease_parameters"),
            size: parameters.crease_count.into() * 4 * 2, /* k, d */
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::STORAGE,
            mapped_at_creation: true
        });

        // targetTheta
        let crease_target_theta = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("rtori-buf-crease_target_theta"),
            size: parameters.crease_count.into() * 4 * 1,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::STORAGE,
            mapped_at_creation: true
        });

        // updated at every step
        let crease_fold_angles = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("rtori-buf-crease_fold_angles"),
            size: parameters.crease_count.into() * 4 * 2,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::STORAGE,
            mapped_at_creation: false
        });

        let crease_physics = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("rtori-buf-crease_physics"),
            size: parameters.crease_count.into() * 4 * 4, /* h1, h2, coef1, coef2 */
            usage: wgpu::BufferUsages::STORAGE,
            mapped_at_creation: false
        });

        let face_nominal_triangles = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("rtori-buf-face_nominal_triangles"),
            size: parameters.face_count.into() * 4 * 3 /* angle a, b, c */,
            usage: wgpu::BufferUsages::STORAGE,
            mapped_at_creation: true
        });

        let node_creases = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("rtori-buf-node_creases"),
            size: parameters.node_crease_count.into() * (1 * 2 /* u16 */ +  1 * 2 /* u8 */),
            usage: wgpu::BufferUsages::STORAGE,
            mapped_at_creation: true
        });

        let node_faces = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("rtori-buf-node_faces"),
            size: parameters.node_face_count.into() * (1 * 2 /* u16 */),
            usage: wgpu::BufferUsages::STORAGE,
            mapped_at_creation: true
        });

        let node_beams = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("rtori-buf-node_beals"),
            size: parameters.node_beam_count.into() * (3 * 4 /* K, d, length */ + 1 * 2 /* other node index */),
            usage: wgpu::BufferUsages::STORAGE,
            mapped_at_creation: true
        });


        /* Bind group layout */
        // See https://toji.dev/webgpu-best-practices/bind-groups.html
        /*
        The WebGPU API is technically agnostic to the order that bind groups are declared in and the order that setBindGroup() is called in.
        Placing the camera bind group at @group(2) and the model bind group at @group(0) works as expected.
        However, the underlying native APIs may have preferences about the order that the groups are declared and set in for performance purposes.
        Thus, to ensure the best performance across the board, you should prefer to have @group(0) contain the values that change least frequently between draw/dispatch calls,
        and each subsequent @group index contain data that changes at progressively higher frequencies.
         */


        // When stepping:
        // - set group 0
        // - then:
        //  - set group 1 for all of them
        //  - set group 

    }
}