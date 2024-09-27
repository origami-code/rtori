#![no_std]
#![feature(portable_simd)]
#![feature(iter_array_chunks)]
#![feature(stmt_expr_attributes)]
#![feature(impl_trait_in_assoc_type)]
#![feature(generic_const_exprs)]

use core::simd::{LaneCount, SupportedLaneCount};

use model::ModelSizes;
#[macro_use]
extern crate static_assertions;
mod kernels;
mod model;
mod process;
mod simd_atoms;

pub struct ReadableRunner<'backer, const L: usize>

where
    LaneCount<L>: SupportedLaneCount {
    state: model::State<'backer, L>
}

pub struct StandbyRunner<'backer, const L: usize>
where
    LaneCount<L>: SupportedLaneCount
{
    state: model::State<'backer, L>
}

impl<'backer, const L: usize> StandbyRunner<'backer, L> where
    LaneCount<L>: SupportedLaneCount,
    simba::simd::Simd<core::simd::Simd<f32, L>>: simba::scalar::RealField,
{
    pub fn step(mut self) -> ReadableRunner<'backer, L> {
        let input = process::ReadOnlyInput {
            node_geometry: &self.state.node_geometry,
            node_positions_unchanging: &self.state.node_positions_unchanging,
            node_external_forces: &self.state.node_external_forces,
            node_mass: &self.state.node_mass,
            node_fixed: &self.state.node_fixed,
            node_position_offset: &self.state.node_position_offset.back,
            node_velocity: &self.state.node_velocity.back,
            crease_face_indices: &self.state.crease_face_indices,
            crease_neighbourhoods: &self.state.crease_neighbourhoods,
            crease_k: &self.state.crease_k,
            crease_target_fold_angle: &self.state.crease_target_fold_angle,
            crease_fold_angle: &self.state.crease_fold_angle.back,
            face_indices: &self.state.face_indices,
            face_nominal_angles: &self.state.face_nominal_angles,
            node_crease_crease_indices: &self.state.node_crease_crease_indices,
            node_crease_node_number: &self.state.node_crease_node_number,
            node_beam_node_index: &self.state.node_beam_node_index,
            node_beam_length: &self.state.node_beam_length,
            node_beam_neighbour_index: &self.state.node_beam_neighbour_index,
            node_beam_k: &self.state.node_beam_k,
            node_beam_d: &self.state.node_beam_d,
            node_face_node_index: &self.state.node_face_node_index,
            node_face_face_index: &self.state.node_face_face_index,
            crease_percentage: self.state.crease_percentage,
            dt: self.state.dt,
            face_stiffness: self.state.face_stiffness,
        };

        let mut scratch = process::ScratchInput {
            crease_physics: &mut self.state.crease_physics,
            face_normals: &mut self.state.face_normals,
            node_crease_forces: &mut self.state.node_crease_forces,
            node_beam_forces: &mut self.state.node_beam_forces,
            node_beam_error: &mut self.state.node_beam_error,
            node_face_forces: &mut self.state.node_face_forces,
            node_face_error: &mut self.state.node_face_error
        };

        let mut memorable = process::MemorableInput {
            crease_fold_angle: &mut self.state.crease_fold_angle.front
        };

        let it = process::process(
            &input,
            &mut scratch,
            &mut memorable
        );

        let position_dest = &mut self.state.node_position_offset.front;
        assert!(it.len() <= position_dest.len());

        let velocity_dest = &mut self.state.node_velocity.front;
        assert!(it.len() <= velocity_dest.len());

        let error_dest = &mut self.state.node_error;
        assert!(it.len() <= error_dest.len());


        for (i, output) in it.enumerate() {
            position_dest[i] = output.position_offset;
            velocity_dest[i] = output.velocity;
            error_dest[i] = output.error;
        }

        ReadableRunner{state: self.state}
    }

    pub fn allocate(input: &'backer mut [u8], sizes: ModelSizes) -> Result<Self, ()> {
        
        todo!()
    }
}