use core::simd::cmp::SimdPartialEq;
use core::simd::cmp::SimdPartialOrd;
use core::simd::LaneCount;
use core::simd::SupportedLaneCount;

use super::algebra::algebrize;
use crate::kernels::operations::gather::{gather_f32, gather_scalar, gather_vec3f_1};
use crate::model::CreaseFaceIndices;
use crate::model::CreasesPhysicsLens;
use crate::simd_atoms::*;

#[derive(Debug)]
pub struct PerNodeCreaseInput<'backer, const L: usize>
where
    LaneCount<L>: SupportedLaneCount,
{
    /* per node-crease */
    pub node_crease_indices: &'backer [SimdU32<L>],
    pub node_crease_node_number: &'backer [SimdU32<L>],

    /* per-crease */
    pub crease_fold_angles: &'backer [SimdF32<L>],
    pub crease_physics: &'backer [CreasesPhysicsLens<L>],
    pub crease_k: &'backer [SimdF32<L>],
    //pub crease_d: &'backer [SimdF32<L>],
    pub crease_target_fold_angle: &'backer [SimdF32<L>],
    pub crease_face_indices: &'backer [CreaseFaceIndices<L>],

    /* per-face */
    pub face_indices: &'backer [SimdVec3U<L>],
    pub face_normals: &'backer [SimdVec3F<L>],

    /* parameters */
    pub crease_percentage: f32,
}

#[inline]
pub fn calculate_node_crease_forces<'a, const L: usize>(
    inputs: &'a PerNodeCreaseInput<'a, L>,
) -> impl ExactSizeIterator<Item = SimdVec3F<L>> + use<'a, L>
where
    LaneCount<L>: SupportedLaneCount,
    simba::simd::Simd<core::simd::Simd<f32, L>>: nalgebra::SimdRealField,
{
    itertools::izip!(inputs.node_crease_indices, inputs.node_crease_node_number).map(
        |(crease_indices, node_number)| {
            let [
                crease_current_fold_angle,
                crease_k,
                //crease_d,
                crease_target_fold_angle
            ] = gather_f32([
                    &inputs.crease_fold_angles,
                    &inputs.crease_k,
                    //&inputs.crease_d,
                    &inputs.crease_target_fold_angle
                ], *crease_indices);

            let crease_physics = CreasesPhysicsLens::gather(inputs.crease_physics, *crease_indices);

            let invalid_physics = crease_physics.simd_is_invalid();
            if invalid_physics.all() {
                // Skip
                return [
                    SimdF32::splat(0.0),
                    SimdF32::splat(0.0),
                    SimdF32::splat(0.0),
                ];
            }

            let crease_percentage_splat = SimdF32::splat(inputs.crease_percentage);
            let adjusted_target_fold_angles = crease_target_fold_angle * crease_percentage_splat;
            let angular_force =
                crease_k * (adjusted_target_fold_angles - crease_current_fold_angle);
            /* 2025-01-15 */
 // println!("Target Fold Angle: {adjusted_target_fold_angles:?} / Angular Force: {angular_force:?}");
 // Now it's time to load the geometry
            let crease_face_indices =
                CreaseFaceIndices::gather(&inputs.crease_face_indices, *crease_indices);

            let crease_reaction_mask = node_number.simd_ge(SimdU32::splat(2));

            // For those who are on a crease
            let force_crease_reaction = {
                let face_indices_a = crease_face_indices.0[0];
                let normal_a = gather_vec3f_1(inputs.face_normals, face_indices_a);

                let face_indices_b = crease_face_indices.0[1];
                let normal_b = gather_vec3f_1(inputs.face_normals, face_indices_b);

                let node_number_is_3 = node_number.simd_eq(SimdU32::splat(3));

                let coef_a = node_number_is_3.select(
                    SimdF32::splat(1.0) - crease_physics.a_coef,
                    crease_physics.a_coef,
                );

                let coef_b = node_number_is_3.select(
                    SimdF32::splat(1.0) - crease_physics.b_coef,
                    crease_physics.b_coef,
                );

                let side = #[inline]
                |normal, coef, height| {
                    algebrize(normal).scale(simba::simd::Simd(coef / height))
                };

                (side(normal_a, coef_a, crease_physics.a_height)
                    + side(normal_b, coef_b, crease_physics.b_height))
                .scale(simba::simd::Simd(SimdF32::splat(-1.0) * angular_force))
            };

            // For those who are off the crease (the complementary nodes)
            let force_complementary = {
                let face_is_1 = node_number.simd_eq(SimdU32::splat(1));

                let face_indices =
                    face_is_1.select(crease_face_indices.0[1], crease_face_indices.0[0]);
                let normals = gather_vec3f_1(&inputs.face_normals, face_indices);

                let moment_arm = face_is_1.select(crease_physics.b_height, crease_physics.a_height);
                // /* 2025-01-15 */ println!("ca_per_node_crease: moment_arm: {moment_arm:?}, face_is_1: {face_is_1:?}, b_height: {:?}, a_height: {:?}", crease_physics.b_height, crease_physics.a_height);

                algebrize(normals).scale(simba::simd::Simd(angular_force / moment_arm))
            };

            let force = [
                crease_reaction_mask.select(force_crease_reaction.x.0, force_complementary.x.0),
                crease_reaction_mask.select(force_crease_reaction.y.0, force_complementary.y.0),
                crease_reaction_mask.select(force_crease_reaction.z.0, force_complementary.z.0),
            ];

            let force_selected = super::operations::select_n(
                invalid_physics,
                [
                    SimdF32::splat(0.0),
                    SimdF32::splat(0.0),
                    SimdF32::splat(0.0),
                ],
                force,
            );

            /* 2025-01-15 */
            /*println!("ca_per_node_crease:
            force selected: {force_selected:?}
            invalid_physics: {invalid_physics:?}
            crease_reaction_mask: {crease_reaction_mask:?}
            force_crease_reaction: {force_crease_reaction:?}
            force_other: {force_other:?}");*/

            super::operations::debug::check_nans_simd_vec_msg(
                force_selected,
                "ca_per_node_crease",
                "force_selected",
            );

            force_selected
        },
    )
}
