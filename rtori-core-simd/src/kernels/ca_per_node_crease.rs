use core::simd::cmp::SimdPartialOrd;
use core::simd::cmp::SimdPartialEq;

use super::algebra::algebrize;
use crate::kernels::operations::gather::{gather_f32, gather_scalar, gather_vec3f_1};
use crate::model::{CreaseGeometries, CreasePhysics};
use crate::{model::CreasesPhysicsLens, simd_atoms::*};

pub struct PerNodeCreaseInput<'backer> {
    /* per-crease */
    pub crease_fold_angles: &'backer [SimdF32],
    pub crease_physics: &'backer CreasePhysics<'backer>,
    pub crease_k: &'backer [SimdF32],
    pub crease_d: &'backer [SimdF32],
    pub crease_target_fold_angle: &'backer [SimdF32],
    pub crease_face_indices: [&'backer [SimdU32]; 2],

    /* per-face */
    pub face_indices: &'backer [SimdVec3U],
    pub face_normals: &'backer [SimdVec3F],

    /* per node-crease */
    pub node_crease_indices: &'backer [SimdU32],
    pub node_crease_node_number: &'backer [SimdU32],

    /* parameters */
    pub crease_percentage: f32
}

const TAU: f32 = 6.283185307179586476925286766559;

pub fn calculate_node_crease_forces<'a>(
    inputs: &'a PerNodeCreaseInput<'a>,
) -> impl ExactSizeIterator<Item = SimdVec3F> + use<'a> {
    itertools::izip!(
        inputs.node_crease_indices,
        inputs.node_crease_node_number
    ).map(|(crease_indices, node_number)| {
        let [
            crease_current_fold_angle,
            crease_physics_a_height,
            crease_physics_a_coef,
            crease_physics_b_height,
            crease_physics_b_coef,
            crease_k,
            crease_d,
            crease_target_fold_angle    
        ] = gather_f32([
                &inputs.crease_fold_angles,
                &inputs.crease_physics.a_height,
                &inputs.crease_physics.a_coef,
                &inputs.crease_physics.b_height,
                &inputs.crease_physics.b_coef,
                &inputs.crease_k,
                &inputs.crease_d,
                &inputs.crease_target_fold_angle
            ], *crease_indices);

        let invalid_physics = crease_physics_a_height.simd_le(SimdF32::splat(0.0));
        if invalid_physics.all() {
            // Skip
            return [
                SimdF32::splat(0.0),
                SimdF32::splat(0.0),
                SimdF32::splat(0.0)
            ];
        }

        let crease_percentage_splat = SimdF32::splat(inputs.crease_percentage);
        let adjusted_target_fold_angles = crease_target_fold_angle * crease_percentage_splat;
        let angular_force = crease_k * (adjusted_target_fold_angles - crease_current_fold_angle);
        
        // Now it's time to load the geometry
        let crease_face_indices = gather_scalar(
            [
                inputs.crease_face_indices[0],
                inputs.crease_face_indices[1]
            ],
            *crease_indices
        );

        let crease_reaction_mask = node_number.simd_ge(SimdU32::splat(2));

        // For those who are on a crease
        let force_crease_reaction = {
            let face_indices_a = crease_face_indices[0];
            let normal_a = gather_vec3f_1(inputs.face_normals, face_indices_a);

            let face_indices_b = crease_face_indices[1];
            let normal_b = gather_vec3f_1(inputs.face_normals, face_indices_b);
            
            let node_number_is_3 = node_number.simd_eq(SimdU32::splat(3));

            let coef_a = node_number_is_3.select(
                SimdF32::splat(1.0) - crease_physics_a_coef,
                crease_physics_a_coef
            );
            
            let coef_b = node_number_is_3.select(
                SimdF32::splat(1.0) - crease_physics_b_coef,
                crease_physics_b_coef
            );

            (
                algebrize(normal_a).scale(simba::simd::Simd(coef_a / crease_physics_a_height))
                + algebrize(normal_b).scale(simba::simd::Simd(coef_b / crease_physics_b_height))
            ).scale(simba::simd::Simd(SimdF32::splat(-1.0) * angular_force))
        };

        // For those who are off the crease
        let force_other = {
            let is_1 = node_number.simd_eq(SimdU32::splat(1));
            let face_indices = is_1.select(
                crease_face_indices[1],
                crease_face_indices[0]
            );
            let normals = gather_vec3f_1(&inputs.face_normals, face_indices);
            
            let moment_arm = is_1.select(
                crease_physics_b_coef,
                crease_physics_a_coef
            );

            algebrize(normals).scale(simba::simd::Simd(angular_force / moment_arm))
        };
        
        let force = [
            crease_reaction_mask.select(force_crease_reaction.x.0, force_other.x.0),
            crease_reaction_mask.select(force_crease_reaction.y.0, force_other.y.0),
            crease_reaction_mask.select(force_crease_reaction.z.0, force_other.z.0)
        ];

        force
    })
}