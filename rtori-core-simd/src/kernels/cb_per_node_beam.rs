use core::simd::LaneCount;
use core::simd::SupportedLaneCount;
use nalgebra::RealField;
use nalgebra::SimdComplexField;
use nalgebra::SimdValue;

use super::algebra::algebrize;
use super::operations::gather::gather_vec3f;
use crate::model::NodeBeamSpec;
use crate::simd_atoms::*;

#[derive(Debug)]
pub struct PerNodeBeamInput<'backer, const L: usize>
where
    LaneCount<L>: SupportedLaneCount,
{
    /* per-node-beam */
    pub beam_spec: &'backer [NodeBeamSpec<L>],
    pub beam_length: &'backer [SimdF32<L>],
    pub beam_k: &'backer [SimdF32<L>],
    pub beam_d: &'backer [SimdF32<L>],

    pub node_positions_unchanging: &'backer [SimdVec3F<L>],
    pub node_positions_offset: &'backer [SimdVec3F<L>],
    pub node_velocity: &'backer [SimdVec3F<L>],
}

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct PerNodeBeamOutput<const L: usize>
where
    LaneCount<L>: SupportedLaneCount,
{
    pub force: SimdVec3F<L>,
    pub error: SimdF32<L>,
}

pub fn calculate_node_beam_forces<'a, const L: usize>(
    inputs: &'a PerNodeBeamInput<'a, L>,
) -> impl ExactSizeIterator<Item = PerNodeBeamOutput<L>> + use<'a, L>
where
    LaneCount<L>: SupportedLaneCount,
    simba::simd::Simd<core::simd::Simd<f32, L>>: nalgebra::SimdRealField,
{
    itertools::izip!(
        inputs.beam_spec,
        inputs.beam_length,
        inputs.beam_k,
        inputs.beam_d
    )
    .map(move |(beam_spec, beam_length, beam_k, beam_d)| {
        // /*2024-10-11*/ println!("per_node_beam: spec: {beam_spec:?}, length: {beam_length:?}, k: {beam_k:?}, d: {beam_d:?}");

        let [node_position_unchanging, node_position_offset, node_velocity] = gather_vec3f(
            [
                &inputs.node_positions_unchanging,
                &inputs.node_positions_offset,
                &inputs.node_velocity,
            ],
            beam_spec.node_indices,
        )
        .map(algebrize);

        let [neighbour_position_unchanging, neighbour_position_offset, neighbour_velocity] =
            gather_vec3f(
                [
                    &inputs.node_positions_unchanging,
                    &inputs.node_positions_offset,
                    &inputs.node_velocity,
                ],
                beam_spec.neighbour_indices,
            )
            .map(algebrize);
        super::operations::debug::check_nans_simd_vec_msg([neighbour_position_unchanging.x.0, neighbour_position_unchanging.y.0, neighbour_position_unchanging.z.0], "cb_per_node_beam", "neighbour_position_unchanging");

        // Calculate delta p
        let (delta_p, error) = {
            let nominal_distance = neighbour_position_unchanging - node_position_unchanging;
            super::operations::debug::check_nans_simd_vec_msg([nominal_distance.x.0, nominal_distance.y.0, nominal_distance.z.0], "cb_per_node_beam", "nominal_distance");

            // /*2024-10-11*/ println!("per_node_beam: nominal_distance: {nominal_distance:?}");
            let delta_p_uncorrected = (neighbour_position_offset - node_position_offset) + nominal_distance;
            super::operations::debug::check_nans_simd_vec_msg([delta_p_uncorrected.x.0, delta_p_uncorrected.y.0, delta_p_uncorrected.z.0], "cb_per_node_beam", "delta_p_uncorrected");

            use std::simd::cmp::SimdPartialEq as _;

            let delta_p_length = delta_p_uncorrected.norm();
            let mask = delta_p_length.0.simd_ne(core::simd::Simd::splat(0.0f32));
            let delta_p_length_corrected = simba::simd::Simd(mask.select(delta_p_length.0, core::simd::Simd::splat(1.0f32)));

            super::operations::debug::check_nans_simd_msg(delta_p_length.0, "cb_per_node_beam", "delta_p_length");
            // /*2024-10-11*/ println!("per_node_beam: delta_p pre-correction: {delta_p:?}, beam_length {beam_length:?}, delta_p_length {delta_p_length:?}");
            super::operations::debug::check_nans_simd_msg(*beam_length, "cb_per_node_beam", "beam_length");

            let delta_p = delta_p_uncorrected * (simba::simd::Simd(*beam_length) /delta_p_length_corrected);
            let error = ((simba::simd::Simd(*beam_length) / delta_p_length_corrected)
                + simba::simd::Simd(SimdF32::splat(-1.0f32)))
            .simd_abs();
            (delta_p, error)
        };
        super::operations::debug::check_nans_simd_vec_msg([delta_p.x.0, delta_p.y.0, delta_p.z.0], "cb_per_node_beam", "delta_p");
        super::operations::debug::check_nans_simd_msg(error.0, "cb_per_node_beam", "error");

        // /*2024-10-11*/ println!("per_node_beam delta_p: {delta_p:?}, error: {error:?}");

        // Calculate delta v
        let delta_v = neighbour_velocity - node_velocity;
        super::operations::debug::check_nans_simd_vec_msg([delta_v.x.0, delta_v.y.0, delta_v.z.0], "cb_per_node_beam", "delta_v");

        // /*2024-10-11*/ println!("per_node_beam delta_v: {delta_v:?}");
        // Calculate resulting force
        let force =
            delta_p.scale(simba::simd::Simd(*beam_k)) + delta_v.scale(simba::simd::Simd(*beam_d));
        let force_output = [force.x.0, force.y.0, force.z.0];

        super::operations::debug::check_nans_simd_vec_msg(force_output, "cb_per_node_beam", &format!("force_output: d_p * k + d_v * d = {delta_p:?} * {beam_k:?} + {delta_v:?} * {beam_d:?}"));

        PerNodeBeamOutput {
            force: force_output,
            error: error.0,
        }
    })
}
