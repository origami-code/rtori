use core::ops::BitOr;
use core::simd::cmp::SimdPartialEq;
use core::simd::cmp::SimdPartialOrd;
use nalgebra::SimdComplexField;
use nalgebra::SimdValue;

use super::algebra::algebrize;
use super::operations::gather::gather_vec3f;
use super::position;
use crate::kernels::operations::gather::{gather_f32, gather_scalar, gather_vec3f_1};
use crate::model::{CreaseGeometryLens, CreasesPhysicsLens};
use crate::simd_atoms::*;

pub struct PerNodeFaceInput<'backer> {
    pub node_face_node_index: &'backer [SimdU32],
    pub node_face_face_index: &'backer [SimdU32],
    pub node_face_count: usize,

    pub node_positions_unchanging: &'backer [SimdVec3F],
    pub node_positions_offset: &'backer [SimdVec3F],
    pub node_velocity: &'backer [SimdVec3F],

    pub face_node_indices: &'backer [SimdVec3U],
    pub face_normals: &'backer [SimdVec3F],
    pub face_nominal_angles: &'backer [SimdVec3F],

    pub face_stiffness: f32,
}

const TAU: f32 = 6.283185307179586476925286766559;
const TOL: f32 = 0.0000001;
pub fn calculate_node_face_forces<'a>(
    inputs: &'a PerNodeFaceInput<'a>,
) -> impl ExactSizeIterator<Item = (SimdVec3F, SimdF32)> + use<'a> {
    let tol = simba::simd::Simd(SimdF32::splat(TOL));
    let face_stiffness = simba::simd::Simd(SimdF32::splat(inputs.face_stiffness));
    let zero = simba::simd::Simd(SimdF32::splat(0.0));
    let zero_force = nalgebra::Vector3::new(zero, zero, zero);

    itertools::izip!(inputs.node_face_node_index, inputs.node_face_face_index).map(
        move |(node_indices, face_indices)| {
            use super::operations::select;

            let face_vertex_indices =
                super::gather::gather_vec3([&inputs.face_node_indices], *face_indices)[0];

            // We have the node indices, so we don't need to do that weird magic
            let a = position::get_positions_for_indices(
                &inputs.node_positions_unchanging,
                &inputs.node_positions_offset,
                face_vertex_indices[0],
            );
            let b = position::get_positions_for_indices(
                &inputs.node_positions_unchanging,
                &inputs.node_positions_offset,
                face_vertex_indices[1],
            );
            let c = position::get_positions_for_indices(
                &inputs.node_positions_unchanging,
                &inputs.node_positions_offset,
                face_vertex_indices[2],
            );

            let ab = b - a;
            let ac = c - a;
            let bc = c - b;

            let ab_length = ab.norm();
            let ac_length = ac.norm();
            let bc_length = bc.norm();

            // TODO Skip if lower than tolerance

            let ab = ab / ab_length;
            let ac = ac / ac_length;
            let bc = bc / bc_length;

            let angles = [
                ab.dot(&ac).simd_acos(),
                simba::simd::Simd(SimdF32::splat(-1.0)) * ab.dot(&bc),
                ac.dot(&bc),
            ];

            let [normal, nominal_angles] = super::gather::gather_vec3f(
                [&inputs.face_normals, &inputs.face_nominal_angles],
                *face_indices,
            );

            let angles_diff =
                algebrize(nominal_angles) - nalgebra::Vector3::new(angles[0], angles[1], angles[2]);
            let angles_diff = angles_diff.scale(face_stiffness);

            let is_a = node_indices.simd_eq(face_vertex_indices[0]);
            let is_b = node_indices.simd_eq(face_vertex_indices[1]);
            let is_c = node_indices.simd_eq(face_vertex_indices[2]);

            let left = select(is_b, ab, ac);
            let left_length = simba::simd::Simd(is_b.select(ab_length.0, ac_length.0));

            let right = select(is_a, ab, bc);
            let right_length = simba::simd::Simd(is_a.select(ab_length.0, bc_length.0));

            let normal = algebrize(normal);
            let cross_left = normal.cross(&left) / left_length;
            let cross_right = normal.cross(&right) / right_length;

            let force_a = select(
                is_a,
                {
                    let mut force = zero_force;
                    force -= (cross_left - cross_right) * angles_diff.x;
                    // TODO: face strain
                    force -= cross_right * angles_diff.y;
                    force += cross_left * angles_diff.z;
                    force
                },
                zero_force,
            );

            let force_b = select(
                is_b,
                {
                    let mut force = zero_force;
                    force -= left * angles_diff.x;
                    force += (left + right) * angles_diff.y;
                    // TODO: face strain
                    force -= right * angles_diff.z;
                    force
                },
                zero_force,
            );

            let force_c = select(
                is_c,
                {
                    let mut force = zero_force;
                    force += left * angles_diff.x;
                    force -= right * angles_diff.y;
                    force += (right - left) * angles_diff.z;
                    force
                },
                zero_force,
            );

            let force = force_a + force_b + force_c;
            let error = zero;
            ([force.x.0, force.y.0, force.z.0], error.0)
        },
    )
}
