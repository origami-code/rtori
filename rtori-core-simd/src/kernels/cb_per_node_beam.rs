use core::ops::BitOr;
use core::simd::cmp::SimdPartialOrd;
use core::simd::cmp::SimdPartialEq;
use nalgebra::SimdComplexField;
use nalgebra::SimdValue;

use super::algebra::algebrize;
use super::operations::gather::gather_vec3f;
use super::position;
use crate::kernels::operations::gather::{gather_f32, gather_scalar, gather_vec3f_1};
use crate::model::{CreaseGeometries, CreasePhysics};
use crate::{model::CreasesPhysicsLens, simd_atoms::*};

pub struct PerNodeBeamInput<'backer> {
    pub node_positions_unchanging: &'backer [SimdVec3F],
    pub node_positions_offset: &'backer [SimdVec3F],
    pub node_velocity: &'backer [SimdVec3F],

    /* per-node-beam */
    pub beam_node_index: &'backer [SimdU32],
    pub beam_k: &'backer [SimdF32],
    pub beam_d: &'backer [SimdF32],
    pub beam_length: &'backer [SimdF32],
    pub beam_neighbour_index: &'backer [SimdU32]
}

const TAU: f32 = 6.283185307179586476925286766559;

pub fn calculate_node_beam_forces<'a>(
    inputs: &'a PerNodeBeamInput<'a>,
) -> impl ExactSizeIterator<Item = (SimdVec3F, SimdF32)> + use<'a> {
    itertools::izip!(
        inputs.beam_node_index,
        inputs.beam_k,
        inputs.beam_d,
        inputs.beam_length,
        inputs.beam_neighbour_index
    ).map(move |(
        beam_node_indices,
        beam_k,
        beam_d,
        beam_length,
        beam_neighbour_index
    )| {
        let [
            node_position_unchanging,
            node_position_offset,
            node_velocity
        ] = gather_vec3f([
            &inputs.node_positions_unchanging,
            &inputs.node_positions_offset,
            &inputs.node_velocity
        ], *beam_node_indices).map(algebrize);

        let [
            neighbour_position_unchanging,
            neighbour_position_offset,
            neighbour_velocity
        ] = gather_vec3f(
            [
                &inputs.node_positions_unchanging,
                &inputs.node_positions_offset,
                &inputs.node_velocity
            ],
            *beam_neighbour_index
        ).map(algebrize);

        // Calculate delta p
        let (delta_p, error) = {


            let nominal_distance = neighbour_position_unchanging - node_position_unchanging;

            let delta_p = (neighbour_position_offset - node_position_offset) + nominal_distance;
            let delta_p_length = delta_p.norm();
            let delta_p = delta_p * (simba::simd::Simd(*beam_length) / delta_p_length);
            let error = ((simba::simd::Simd(*beam_length) / delta_p_length) + simba::simd::Simd::splat(-1.0)).simd_abs();
            (delta_p, error)
        };

        // Calculate delta v
        let delta_v = neighbour_velocity - node_velocity;
        
        // Calculate resulting force
        let force = delta_p.scale(simba::simd::Simd(*beam_k)) + delta_v.scale(simba::simd::Simd(*beam_d));
        let force_output = [
            force.x.0,
            force.y.0,
            force.z.0
        ];

        (force_output, error.0)
    })
}