use core::simd::{LaneCount, SupportedLaneCount};

use super::{algebra::algebrize, gather::gather_vec3f_1, position};
use crate::model::{CreaseFaceIndices, CreaseNeighbourhood};
use crate::simd_atoms::*;
use nalgebra::{SimdComplexField, SimdRealField};
use nalgebra::{SimdPartialOrd, SimdValue};

#[derive(Debug)]
pub struct PerCreaseFoldAngleInput<'backer, const L: usize>
where
    LaneCount<L>: SupportedLaneCount,
{
    pub crease_face_indices: &'backer [CreaseFaceIndices<L>],
    pub crease_neighbourhoods: &'backer [CreaseNeighbourhood<L>],
    pub crease_fold_angle: &'backer [SimdF32<L>],

    pub node_positions_offset: &'backer [SimdVec3F<L>],
    pub node_positions_unchanging: &'backer [SimdVec3F<L>],

    pub face_normals: &'backer [SimdVec3F<L>],
}

pub struct PerCreaseFoldAngleOutput<'backer, const L: usize>
where
    LaneCount<L>: SupportedLaneCount,
{
    crease_fold_angle: &'backer mut [SimdF32<L>],
}

pub fn calculate_crease_fold_angles<'a, const L: usize>(
    inputs: &'a PerCreaseFoldAngleInput<'a, L>,
) -> impl ExactSizeIterator<Item = SimdF32<L>> + use<'a, L>
where
    LaneCount<L>: SupportedLaneCount,
    simba::simd::Simd<core::simd::Simd<f32, L>>:
        num_traits::Num + num_traits::NumAssign + SimdComplexField + SimdRealField,
{
    itertools::izip!(
        inputs.crease_face_indices,
        inputs.crease_neighbourhoods,
        inputs.crease_fold_angle
    )
    .map(|(face_indices, neighbourhoods, previous_fold_angles)| {

        let g = #[inline(always)]
        |face_index| {
            let face_indices_a = face_indices.0[face_index];
            let normals_a = gather_vec3f_1(&inputs.face_normals, face_indices_a);
            let normals_a = algebrize(normals_a);
            normals_a
        };

        let normals_a = g(0);
        let normals_b = g(1);

        let normals_dot_unclamped = normals_a.dot(&normals_b);
        let normals_dot_clamped = simba::simd::Simd::simd_clamp(
            normals_dot_unclamped,
            simba::simd::Simd(SimdF32::splat(-1.0)),
            simba::simd::Simd(SimdF32::splat(1.0)),
        );

        let get_adjacent = |face_index| {
            position::get_positions_for_indices(
                inputs.node_positions_unchanging,
                inputs.node_positions_offset,
                neighbourhoods.adjacent_node_indices[face_index],
            )
        };

        let vertex_a = get_adjacent(0);
        let vertex_b = get_adjacent(1);

        let ab = vertex_b - vertex_a;
                println!("per_crease_fold_angle: vertex_a {vertex_a:?} vertex_b {vertex_b:?} ab {ab:?} neighbourhoods {neighbourhoods:?}");
        let crease_vector = ab.normalize();

        let x = normals_dot_clamped;
        let y = normals_a.cross(&crease_vector).dot(&normals_b);
        let fold_angle = simba::simd::Simd::simd_atan2(y, x);
            println!("per_crease_fold_angle uncorrected fold angle {fold_angle:?} (y: {y:?}, x: {x:?}, crease_vector: {crease_vector:?})");


        if true {
            let zero = simba::simd::Simd(SimdF32::splat(0.0));
            let tau = simba::simd::Simd(SimdF32::splat(core::f32::consts::TAU));
            let diff = fold_angle - simba::simd::Simd(*previous_fold_angles);

            let under = diff.simd_le(simba::simd::Simd(SimdF32::splat(-5.0)));
            let under_diff = tau.select(under, zero);

            let over = diff.simd_ge(simba::simd::Simd(SimdF32::splat(5.0)));
            let over_diff = (-tau).select(over, zero);

            let corrected = simba::simd::Simd(*previous_fold_angles) + under_diff + over_diff;
            println!("per_crease_fold_angle Derived fold angle {corrected:?}");

            corrected
        } else {
            fold_angle
        }
        .0
    })
}
