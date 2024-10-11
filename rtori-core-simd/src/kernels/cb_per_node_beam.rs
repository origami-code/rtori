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
        println!("per_node_beam: spec: {beam_spec:?}, length: {beam_length:?}, k: {beam_k:?}, d: {beam_d:?}");

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

        // Calculate delta p
        let (delta_p, error) = {
            let nominal_distance = neighbour_position_unchanging - node_position_unchanging;
            println!("per_node_beam: nominal_distance: {nominal_distance:?}");
            let delta_p = (neighbour_position_offset - node_position_offset) + nominal_distance;

            let delta_p_length = delta_p.norm();
                println!("per_node_beam: delta_p pre-correction: {delta_p:?}, beam_length {beam_length:?}, delta_p_length {delta_p_length:?}");

            let delta_p = delta_p * (simba::simd::Simd(*beam_length) / delta_p_length);
            let error = ((simba::simd::Simd(*beam_length) / delta_p_length)
                + simba::simd::Simd(SimdF32::splat(-1.0f32)))
            .simd_abs();
            (delta_p, error)
        };

        println!("per_node_beam delta_p: {delta_p:?}, error: {error:?}");

        // Calculate delta v
        let delta_v = neighbour_velocity - node_velocity;
        println!("per_node_beam delta_v: {delta_v:?}");
        // Calculate resulting force
        let force =
            delta_p.scale(simba::simd::Simd(*beam_k)) + delta_v.scale(simba::simd::Simd(*beam_d));
        let force_output = [force.x.0, force.y.0, force.z.0];

        PerNodeBeamOutput {
            force: force_output,
            error: error.0,
        }
    })
}
