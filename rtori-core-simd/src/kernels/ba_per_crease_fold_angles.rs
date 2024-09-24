use super::{
    algebra::algebrize,
    gather::{gather_vec3f_1},
    position,
};
use crate::model::CreaseGeometry;
use crate::simd_atoms::*;
use nalgebra::SimdRealField;
use nalgebra::{SimdPartialOrd, SimdValue};

pub struct PerCreaseFoldAngleInput<'backer> {
    node_positions_offset: &'backer [SimdVec3F],
    node_positions_unchanging: &'backer [SimdVec3F],
    face_normals: &'backer [SimdVec3F],
    crease_geometry: &'backer [CreaseGeometry],
    crease_fold_angle: &'backer [SimdF32],
}

pub struct PerCreaseFoldAngleOutput<'backer> {
    crease_fold_angle: &'backer mut [SimdF32],
}

pub fn calculate_crease_fold_angles<'a>(
    inputs: &'a PerCreaseFoldAngleInput<'a>,
) -> impl ExactSizeIterator<Item = SimdF32> + use<'a> {
    itertools::izip!(inputs.crease_geometry, inputs.crease_fold_angle).map(
        |(geometry, previous_fold_angles)| {
            let g = #[inline(always)]
            |face_index| {
                let face_indices_a = geometry.face_indices[face_index];
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
                    geometry.adjacent_node_indices[face_index],
                )
            };

            let vertex_a = get_adjacent(0);
            let vertex_b = get_adjacent(1);

            let ab = vertex_b - vertex_a;
            let crease_vector = ab.normalize();

            let x = normals_dot_clamped;
            let y = normals_a.cross(&crease_vector).dot(&normals_b);
            let fold_angle = simba::simd::Simd::simd_atan2(y, x);

            if true {
                let zero = simba::simd::Simd(SimdF32::splat(0.0));
                let tau = simba::simd::Simd(SimdF32::splat(3.1415 * 2.0));
                let diff = fold_angle - simba::simd::Simd(*previous_fold_angles);

                let under = diff.simd_le(simba::simd::Simd(SimdF32::splat(-5.0)));
                let under_diff = tau.select(under, zero);

                let over = diff.simd_ge(simba::simd::Simd(SimdF32::splat(5.0)));
                let over_diff = (tau * simba::simd::Simd(SimdF32::splat(-1.0))).select(over, zero);

                let corrected = simba::simd::Simd(*previous_fold_angles) + under_diff + over_diff;

                corrected
            } else {
                fold_angle
            }
            .0
        },
    )
}
