#![no_std]
#![feature(portable_simd)]
#![feature(iter_array_chunks)]
#![feature(stmt_expr_attributes)]
#![feature(impl_trait_in_assoc_type)]
#![feature(generic_const_exprs)]
#![feature(const_swap)]
use core::simd::{LaneCount, SupportedLaneCount};

use model::ModelSizes;
#[macro_use]
extern crate static_assertions;
mod kernels;
mod loader;
mod model;
mod process;
mod simd_atoms;

pub struct Runner<'backer, const L: usize = { simd_atoms::PREFERRED_WIDTH }>
where
    LaneCount<L>: SupportedLaneCount,
{
    steps: u64,
    state: model::State<'backer, L>,
}

impl<'backer, const L: usize> Runner<'backer, L>
where
    LaneCount<L>: SupportedLaneCount,
    simba::simd::Simd<core::simd::Simd<f32, L>>: simba::scalar::RealField,
{
    pub fn step(&mut self) -> Result<(), ()> {
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
            node_beam_spec: &self.state.node_beam_spec,
            node_beam_length: &self.state.node_beam_length,
            node_beam_k: &self.state.node_beam_k,
            node_beam_d: &self.state.node_beam_d,
            node_face_spec: &self.state.node_face_spec,
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
            node_face_error: &mut self.state.node_face_error,
        };

        let mut memorable = process::MemorableInput {
            crease_fold_angle: &mut self.state.crease_fold_angle.front,
        };

        let it = process::process(&input, &mut scratch, &mut memorable);

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

        // Swap
        self.state.node_position_offset.swap();
        self.state.node_velocity.swap();
        self.state.crease_fold_angle.swap();

        self.steps += 1;

        Ok(())
    }

    pub fn query_backing_size_requirement(sizes: &ModelSizes) -> usize {
        crate::model::State::required_backing_size(sizes)
    }

    pub fn from_backing_slice(
        sizes: &ModelSizes,
        backing_slice: &'backer mut [u8],
    ) -> Result<(Self, &'backer mut [u8]), usize> {
        crate::model::State::from_slice(sizes, backing_slice)
            .map_err(|_| crate::model::State::required_backing_size(sizes))
            .map(|(state, rest)| (Self { steps: 0, state }, rest))
    }

    pub fn from_allocator_func<'a, F>(
        sizes: &ModelSizes,
        allocator: F,
    ) -> Option<(Self, &'backer mut [u8])>
    where
        F: FnOnce(usize) -> Option<&'backer mut [u8]>,
    {
        // Calculate how much size it would take
        let size = crate::model::State::required_backing_size(sizes);
        let backing_array = allocator(size)?;

        Self::from_backing_slice(sizes, backing_array)
            .expect("backing_array should explicitely be of the right size, or larger")
            .into()
    }

    pub fn load<'a>(&'a mut self) -> impl rtori_os_model::Destination + use<'backer, 'a, L>
    where
        'a: 'backer,
    {
        loader::Loader::new(&mut self.state)
    }

    pub fn read(&self) -> &model::State<L> {
        &self.state
    }

    pub fn step_count(&self) -> u64 {
        self.steps
    }
}
