use core::simd::cmp::SimdPartialEq;

use super::algebra::algebrize;
use crate::simd_atoms::*;

pub struct PerNodeInput<'backer> {
    pub node_positions_offset: &'backer [SimdVec3F],
    pub node_velocity: &'backer [SimdVec3F],

    pub node_external_forces: &'backer [SimdVec3F],
    pub node_mass: &'backer [SimdF32],
    pub node_fixed: &'backer [SimdU32],

    pub node_crease_force: &'backer [SimdVec3F],
    pub node_beam_force: &'backer [SimdVec3F],
    pub node_face_force: &'backer [SimdVec3F],

    pub dt: f32,
}

pub struct PerNodeOutput {
    pub position_offset: SimdVec3F,
    pub velocity: SimdVec3F,
    pub error: SimdF32,
}

pub fn calculate_node_position<'a>(
    inputs: &'a PerNodeInput<'a>,
) -> impl ExactSizeIterator<Item = PerNodeOutput> + use<'a> {
    let zero = simba::simd::Simd(SimdF32::splat(0.0));
    let zero_force = nalgebra::Vector3::new(zero, zero, zero);

    let dt = simba::simd::Simd(SimdF32::splat(inputs.dt));

    itertools::izip!(
        inputs.node_positions_offset,
        inputs.node_velocity,
        inputs.node_external_forces,
        inputs.node_mass,
        inputs.node_fixed,
        inputs.node_crease_force,
        inputs.node_beam_force,
        inputs.node_face_force
    )
    .map(
        move |(
            positions_offset,
            velocity,
            external_forces,
            mass,
            fixed,
            crease_force,
            beam_force,
            face_force,
        )| {
            let force = algebrize(*external_forces)
                + algebrize(*crease_force)
                + algebrize(*beam_force)
                + algebrize(*face_force);

            let velocity_diff = force.scale(dt) / simba::simd::Simd(*mass);

            let velocity_new = algebrize(*velocity) + velocity_diff;
            let is_fixed_mask = fixed.simd_eq(SimdU32::splat(1));

            let position_offset_diff =
                super::operations::select(is_fixed_mask, velocity_new * dt, zero_force);

            let position_offset = algebrize(*positions_offset) + position_offset_diff;

            PerNodeOutput {
                position_offset: [
                    position_offset.x.0,
                    position_offset.y.0,
                    position_offset.z.0,
                ],
                velocity: [velocity_new.x.0, velocity_new.y.0, velocity_new.z.0],
                error: zero.0,
            }
        },
    )
}
