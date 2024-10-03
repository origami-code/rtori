use core::simd::cmp::SimdPartialEq;
use core::simd::LaneCount;
use core::simd::SupportedLaneCount;
use nalgebra::SimdComplexField;

use super::algebra::algebrize;
use super::position;
use crate::model::NodeFaceSpec;
use crate::simd_atoms::*;

#[derive(Debug)]
pub struct PerNodeFaceInput<'backer, const L: usize>
where
    LaneCount<L>: SupportedLaneCount,
{
    pub node_face_spec: &'backer [NodeFaceSpec<L>],

    pub node_positions_unchanging: &'backer [SimdVec3F<L>],
    pub node_positions_offset: &'backer [SimdVec3F<L>],
    pub node_velocity: &'backer [SimdVec3F<L>],

    pub face_node_indices: &'backer [SimdVec3U<L>],
    pub face_normals: &'backer [SimdVec3F<L>],
    pub face_nominal_angles: &'backer [SimdVec3F<L>],

    pub face_stiffness: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct PerNodeFaceOutput<const L: usize>
where
    LaneCount<L>: SupportedLaneCount,
{
    pub force: SimdVec3F<L>,
    pub error: SimdF32<L>,
}

const TAU: f32 = 6.283185307179586476925286766559;
const TOL: f32 = 0.0000001;

pub fn calculate_node_face_forces<'a, const L: usize>(
    inputs: &'a PerNodeFaceInput<'a, L>,
) -> impl ExactSizeIterator<Item = PerNodeFaceOutput<L>> + use<'a, L>
where
    LaneCount<L>: SupportedLaneCount,
    simba::simd::Simd<core::simd::Simd<f32, L>>: nalgebra::SimdRealField,
{
    let tol = simba::simd::Simd(SimdF32::splat(TOL));
    let face_stiffness = simba::simd::Simd(SimdF32::splat(inputs.face_stiffness));
    let zero = simba::simd::Simd(SimdF32::splat(0.0));
    let zero_force = nalgebra::Vector3::new(zero, zero, zero);

    inputs.node_face_spec.iter().map(
        #[inline]
        move |spec| {
            use super::operations::select;

            let face_vertex_indices =
                super::gather::gather_vec3([&inputs.face_node_indices], spec.face_indices)[0];

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
                spec.face_indices,
            );

            let angles_diff =
                algebrize(nominal_angles) - nalgebra::Vector3::new(angles[0], angles[1], angles[2]);
            let angles_diff = angles_diff.scale(face_stiffness);

            let is_a = spec.node_indices.simd_eq(face_vertex_indices[0]);
            let is_b = spec.node_indices.simd_eq(face_vertex_indices[1]);
            let is_c = spec.node_indices.simd_eq(face_vertex_indices[2]);

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

            PerNodeFaceOutput {
                force: [force.x.0, force.y.0, force.z.0],
                error: error.0,
            }
        },
    )
}
