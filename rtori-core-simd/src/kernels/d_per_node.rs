use core::simd::{cmp::SimdPartialEq, LaneCount, SupportedLaneCount};

use nalgebra::{SimdComplexField, SimdRealField};

use super::{algebra::algebrize, operations};
use crate::simd_atoms::*;
use super::operations::debug::ensure_simd;

#[derive(Debug)]
pub struct PerNodeInput<'backer, const L: usize>
where
    LaneCount<L>: SupportedLaneCount,
{
    pub node_positions_offset: &'backer [SimdVec3F<L>],
    pub node_velocity: &'backer [SimdVec3F<L>],

    pub node_external_forces: &'backer [SimdVec3F<L>],
    pub node_mass: &'backer [SimdF32<L>],
    pub node_fixed: &'backer [SimdU32<L>],

    pub node_crease_force: &'backer [SimdVec3F<L>],
    pub node_beam_force: &'backer [SimdVec3F<L>],
    pub node_face_force: &'backer [SimdVec3F<L>],

    pub dt: f32,
}

#[derive(Debug)]
pub struct PerNodeInputLens<'backer, const L: usize>
where
    LaneCount<L>: SupportedLaneCount,
{
    pub positions_offset: &'backer SimdVec3F<L>,
    pub velocity: &'backer SimdVec3F<L>,
    pub external_forces: &'backer SimdVec3F<L>,
    pub mass: &'backer SimdF32<L>,
    pub fixed: &'backer SimdU32<L>,
    pub crease_force: &'backer SimdVec3F<L>,
    pub beam_force: &'backer SimdVec3F<L>,
    pub face_force: &'backer SimdVec3F<L>,
}

impl<'backer, const L: usize> IntoIterator for PerNodeInput<'backer, L>
where
    LaneCount<L>: SupportedLaneCount,
{
    type Item = PerNodeInputLens<'backer, L>;

    type IntoIter = impl ExactSizeIterator<Item = Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
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
                PerNodeInputLens {
                    positions_offset,
                    velocity,
                    external_forces,
                    mass,
                    fixed,
                    crease_force,
                    beam_force,
                    face_force,
                }
            },
        )
    }
}

pub struct PerNodeOutput<const L: usize>
where
    LaneCount<L>: SupportedLaneCount,
{
    pub position_offset: SimdVec3F<L>,
    pub velocity: SimdVec3F<L>,
    pub error: SimdF32<L>,
}

pub fn calculate_node_position<'a, const L: usize>(
    inputs: PerNodeInput<'a, L>,
) -> impl ExactSizeIterator<Item = PerNodeOutput<L>> + use<'a, L>
where
    LaneCount<L>: SupportedLaneCount,
    simba::simd::Simd<core::simd::Simd<f32, L>>:
        num_traits::Num + num_traits::NumAssign + SimdComplexField + SimdRealField,
{
    let zero = simba::simd::Simd(SimdF32::splat(0.0));
    let zero_force = nalgebra::Vector3::new(zero, zero, zero);

    let dt = simba::simd::Simd(SimdF32::splat(inputs.dt));

    inputs.into_iter().map(move |per_node| {
        let valid_input = per_node.mass.simd_ne(SimdF32::splat(0.0));

        let force = algebrize(*per_node.external_forces)
            + algebrize(*per_node.crease_force)
            + algebrize(*per_node.beam_force)
            + algebrize(*per_node.face_force);
        ensure_simd!(force; v3);

        let velocity_diff = force.scale(dt) / simba::simd::Simd(*per_node.mass);
        ensure_simd!(velocity_diff; v3; @mask(valid_input));

        let velocity_new = algebrize(*per_node.velocity) + velocity_diff;
        ensure_simd!(velocity_new; v3; @mask(valid_input));

        let is_fixed_mask = per_node.fixed.simd_eq(SimdU32::splat(1));

        let position_offset_diff =
            super::operations::select(is_fixed_mask, zero_force, velocity_new * dt);

        let position_offset = algebrize(*per_node.positions_offset) + position_offset_diff;
        /* 2025-01-13 */ /*println!("
mass: {:?}
positions: {:?}
force from crease {:?}
force from beam {:?}
force from face {:?}
force (unscaled by dt): {:?}
dt: {:?}", *per_node.mass, position_offset, per_node.crease_force, per_node.beam_force, per_node.face_force, force, dt);*/

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
