use core::simd::{cmp::SimdPartialEq, LaneCount, SupportedLaneCount};

use nalgebra::{geometry, SimdComplexField, SimdRealField};
use simd_common::*;

use super::operations::debug::ensure_simd;
use crate::model::NodeGeometry;

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

    pub node_geometry: &'backer [NodeGeometry<L>],

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
    pub geometry: &'backer NodeGeometry<L>,
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
            self.node_geometry
        )
        .enumerate()
        .map(
            move |(
                _chunk_index,
                (positions_offset, velocity, external_forces, mass, fixed, geometry),
            )| {
                PerNodeInputLens {
                    positions_offset,
                    velocity,
                    external_forces,
                    mass,
                    fixed,
                    geometry,
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

fn calculate_force_subset<const L: usize>(
    range: &crate::model::NodeGeometryRange<L>,
    forces: &[SimdVec3F<L>],
) -> nalgebra::Vector3<simba::simd::Simd<core::simd::Simd<f32, { L }>>>
where
    LaneCount<L>: SupportedLaneCount,
    simba::simd::Simd<core::simd::Simd<f32, L>>:
        num_traits::Num + num_traits::NumAssign + SimdComplexField + SimdRealField,
{
    /* Get the maximum count of node_??? to go through */
    let count = range.count;

    use core::simd::num::SimdUint;
    let count_max = count.reduce_max();

    (0..count_max)
        .into_iter()
        .map(|i| {
            use core::simd::cmp::SimdPartialOrd;

            let simd_i = core::simd::Simd::splat(i);

            let valid = range.count.simd_gt(simd_i);
            let cursor = valid.select(simd_i, core::simd::Simd::splat(0));

            let indices = range.offset + cursor;

            let forces = super::operations::gather::gather_vec3f_1(forces, indices);

            let forces_filtered = super::operations::select_n(
                valid,
                forces,
                [
                    core::simd::Simd::splat(0.0),
                    core::simd::Simd::splat(0.0),
                    core::simd::Simd::splat(0.0),
                ],
            );

            simd_common::convert_nalgebra::to_nalgebra3(forces_filtered)
        })
        .sum()
}

#[tracing::instrument]
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

    let crease_forces = inputs.node_crease_force;
    let beam_forces = inputs.node_beam_force;
    let _face_forces = inputs.node_face_force;

    inputs.into_iter().map(move |per_node| {
        let crease_force = calculate_force_subset(&per_node.geometry.creases, &crease_forces);
        let beam_force = calculate_force_subset(&per_node.geometry.beams, &beam_forces);
        //let face_force = calculate_force_subset(&per_node.geometry.faces, &face_forces);
        let face_force = simd_common::convert_nalgebra::to_nalgebra3([
            core::simd::Simd::splat(0.0),
            core::simd::Simd::splat(0.0),
            core::simd::Simd::splat(0.0),
        ]);
        let valid_input = per_node.mass.simd_ne(SimdF32::splat(0.0));

        let force = simd_common::convert_nalgebra::to_nalgebra3(*per_node.external_forces)
            + crease_force
            + beam_force
            + face_force;
        ensure_simd!(force; v3);

        let velocity_diff = force.scale(dt) / simba::simd::Simd(*per_node.mass);
        ensure_simd!(velocity_diff; v3; @mask(valid_input));

        let velocity_new =
            simd_common::convert_nalgebra::to_nalgebra3(*per_node.velocity) + velocity_diff;
        ensure_simd!(velocity_new; v3; @mask(valid_input));

        let is_fixed_mask = per_node.fixed.simd_eq(SimdU32::splat(1));

        let position_offset_diff =
            super::operations::select(is_fixed_mask, zero_force, velocity_new * dt);

        let position_offset =
            simd_common::convert_nalgebra::to_nalgebra3(*per_node.positions_offset)
                + position_offset_diff;
        /* 2025-01-15 */

        tracing::event!(
            tracing::Level::TRACE,
            "
 mass: {:?}
 positions (diff): {:?}
 positions (new): {:?}
 velocity (diff): {:?}
 velocity (new): {:?}
 force from crease {:?}
 force from beam {:?}
 force from face {:?}
 force (unscaled by dt): {:?}
 dt: {:?}",
            *per_node.mass,
            position_offset_diff,
            position_offset,
            velocity_diff,
            velocity_new,
            crease_force,
            beam_force,
            face_force,
            force,
            dt
        );

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
