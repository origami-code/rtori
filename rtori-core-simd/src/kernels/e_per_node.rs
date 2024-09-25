use core::simd::{cmp::SimdPartialEq, LaneCount, SupportedLaneCount};

use nalgebra::{SimdComplexField, SimdRealField};

use super::{algebra::algebrize, input_iterator::InputIteratorItem};
use crate::simd_atoms::*;

pub struct PerNodeInput<'backer, const L: usize>
where
    LaneCount<L>: SupportedLaneCount,
{
    pub node_count: usize,

    pub node_positions_offset: &'backer [SimdVec3FN<L>],
    pub node_velocity: &'backer [SimdVec3FN<L>],

    pub node_external_forces: &'backer [SimdVec3FN<L>],
    pub node_mass: &'backer [SimdF32N<L>],
    pub node_fixed: &'backer [SimdU32N<L>],

    pub node_crease_force: &'backer [SimdVec3FN<L>],
    pub node_beam_force: &'backer [SimdVec3FN<L>],
    pub node_face_force: &'backer [SimdVec3FN<L>],

    pub dt: f32,
}

pub struct PerNodeInputLens<'backer, const L: usize>
where
    LaneCount<L>: SupportedLaneCount,
{
    pub positions_offset: &'backer SimdVec3FN<L>,
    pub velocity: &'backer SimdVec3FN<L>,
    pub external_forces: &'backer SimdVec3FN<L>,
    pub mass: &'backer SimdF32N<L>,
    pub fixed: &'backer SimdU32N<L>,
    pub crease_force: &'backer SimdVec3FN<L>,
    pub beam_force: &'backer SimdVec3FN<L>,
    pub face_force: &'backer SimdVec3FN<L>,
}

impl<'backer, const L: usize> IntoIterator for &'backer PerNodeInput<'backer, L>
where
    LaneCount<L>: SupportedLaneCount,
{
    type Item = InputIteratorItem<PerNodeInputLens<'backer, L>, L>;

    type IntoIter = impl ExactSizeIterator<Item = Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        let last_chunk_index = self.node_count / L;

        let last_chunk_valid_count = self.node_count % L;
        let last_chunk_mask =
            core::simd::Mask::from_bitmask((1 << (last_chunk_valid_count + 1)) - 1);
        let passthrough = core::simd::Mask::splat(true);

        itertools::izip!(
            self.node_positions_offset,
            self.node_velocity,
            self.node_external_forces,
            self.node_mass,
            self.node_fixed,
            self.node_crease_force,
            self.node_beam_force,
            self.node_face_force
        )
        .enumerate()
        .map(
            move |(
                chunk_index,
                (
                    positions_offset,
                    velocity,
                    external_forces,
                    mass,
                    fixed,
                    crease_force,
                    beam_force,
                    face_force,
                ),
            )| {
                let mask = if (chunk_index < last_chunk_index) {
                    passthrough
                } else {
                    last_chunk_mask
                };

                InputIteratorItem {
                    mask: mask,
                    lens: PerNodeInputLens {
                        positions_offset,
                        velocity,
                        external_forces,
                        mass,
                        fixed,
                        crease_force,
                        beam_force,
                        face_force,
                    },
                }
            },
        )
    }
}

pub struct PerNodeOutput<const L: usize>
where
    LaneCount<L>: SupportedLaneCount,
{
    pub position_offset: SimdVec3FN<L>,
    pub velocity: SimdVec3FN<L>,
    pub error: SimdF32N<L>,
}

pub fn calculate_node_position<'a, const L: usize>(
    inputs: &'a PerNodeInput<'a, L>,
) -> impl ExactSizeIterator<Item = PerNodeOutput<L>> + use<'a, L>
where
    LaneCount<L>: SupportedLaneCount,
    simba::simd::Simd<core::simd::Simd<f32, L>>:
        num_traits::Num + num_traits::NumAssign + SimdComplexField + SimdRealField,
{
    let zero = simba::simd::Simd(SimdF32N::splat(0.0));
    let zero_force = nalgebra::Vector3::new(zero, zero, zero);

    let dt = simba::simd::Simd(SimdF32N::splat(inputs.dt));

    inputs.into_iter().map(move |per_node| {
        let force = algebrize(*per_node.lens.external_forces)
            + algebrize(*per_node.lens.crease_force)
            + algebrize(*per_node.lens.beam_force)
            + algebrize(*per_node.lens.face_force);

        let velocity_diff = force.scale(dt) / simba::simd::Simd(*per_node.lens.mass);

        let velocity_new = algebrize(*per_node.lens.velocity) + velocity_diff;
        let is_fixed_mask = per_node.lens.fixed.simd_eq(SimdU32N::splat(1));

        let position_offset_diff =
            super::operations::select(is_fixed_mask, velocity_new * dt, zero_force);

        let position_offset = algebrize(*per_node.lens.positions_offset) + position_offset_diff;

        PerNodeOutput {
            position_offset: [
                position_offset.x.0,
                position_offset.y.0,
                position_offset.z.0,
            ],
            velocity: [velocity_new.x.0, velocity_new.y.0, velocity_new.z.0],
            error: zero.0,
        }
    })
}
