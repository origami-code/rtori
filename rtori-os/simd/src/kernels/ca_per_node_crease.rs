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

#[tracing::instrument]
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
            tracing::event!(tracing::Level::TRACE, "Crease Percentage: {crease_percentage_splat:?}\n\tTarget Fold Angle:{adjusted_target_fold_angles:?}\n\tCurrent Fold Angle: {crease_current_fold_angle:?}\n\tAngular Force: {angular_force:?}");
 // Now it's time to load the geometry
            let crease_face_indices =
                CreaseFaceIndices::gather(&inputs.crease_face_indices, *crease_indices);

            // Node indices 3 & 4 mean the node is on the crease
            // Node indices 1 & 2 mean the node is complementary
            let crease_reaction_mask = node_number.simd_gt(SimdU32::splat(2));

            // For those who are on a crease (node index 3 or 4)
            let force_crease_reaction = {
                let normal_a = gather_vec3f_1(inputs.face_normals, crease_face_indices.0[0]);
                let normal_b = gather_vec3f_1(inputs.face_normals, crease_face_indices.0[1]);

                let [coef_a, coef_b] = {
                    let node_number_is_3 = node_number.simd_eq(SimdU32::splat(3));

                    let coef_a = node_number_is_3.select(
                        SimdF32::splat(1.0) - crease_physics.a_coef,
                        crease_physics.a_coef,
                    );

                    let coef_b = node_number_is_3.select(
                        SimdF32::splat(1.0) - crease_physics.b_coef,
                        crease_physics.b_coef,
                    );

                    [coef_a, coef_b]
                };

                // Applies the operation (coef / height) * normal
                let side = #[inline]
                |normal, coef, height| {
                    algebrize(normal).scale(simba::simd::Simd(coef / height))
                };

                (side(normal_a, coef_a, crease_physics.a_height)
                    + side(normal_b, coef_b, crease_physics.b_height))
                .scale(simba::simd::Simd(SimdF32::splat(-1.0) * angular_force))
            };

            // For those who are off the crease (the complementary nodes, node indice 1 or 2)
            let force_complementary = {
                let face_is_b = node_number.simd_eq(SimdU32::splat(2));

                let face_indices =
                    face_is_b.select(crease_face_indices.0[1], crease_face_indices.0[0]);
                let normals = gather_vec3f_1(&inputs.face_normals, face_indices);

                let moment_arm = face_is_b.select(crease_physics.b_height, crease_physics.a_height);
                // /* 2025-01-15 */ println!("ca_per_node_crease: moment_arm: {moment_arm:?}, face_is_b: {face_is_b:?}, b_height: {:?}, a_height: {:?}", crease_physics.b_height, crease_physics.a_height);

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
            tracing::event!(tracing::Level::TRACE, "Finished
            crease count: {}
            crease_indices: {crease_indices:?}
            node_number: {node_number:?}
            force selected: {force_selected:?}
            invalid_physics: {invalid_physics:?}
            crease_reaction_mask: {crease_reaction_mask:?}
            force_crease_reaction: {force_crease_reaction:?}
            force_complementary: {force_complementary:?}
            crease_physics: {crease_physics:?}", inputs.node_crease_indices.len() * 4);

            super::operations::debug::check_nans_simd_vec_msg(
                force_selected,
                "ca_per_node_crease",
                "force_selected",
            );

            force_selected
        },
    )
}
