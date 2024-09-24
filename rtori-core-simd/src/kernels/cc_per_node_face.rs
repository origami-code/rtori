use core::ops::BitOr;
use core::simd::cmp::SimdPartialOrd;
use core::simd::cmp::SimdPartialEq;
use nalgebra::SimdComplexField;
use nalgebra::SimdValue;

use super::algebra::algebrize;
use super::operations::gather::gather_vec3f;
use super::position;
use crate::kernels::operations::gather::{gather_f32, gather_scalar, gather_vec3f_1};
use crate::model::{CreaseGeometries, CreasePhysics};
use crate::{model::CreasesPhysicsLens, simd_atoms::*};

pub struct PerNodeFaceInput<'backer> {
    pub node_positions_unchanging: &'backer [SimdVec3F],
    pub node_positions_offset: &'backer [SimdVec3F],
    pub node_velocity: &'backer [SimdVec3F],

    pub face_node_indices: &'backer [SimdVec3U],
    pub face_normals: &'backer [SimdVec3F],
    pub face_nominal_angles: &'backer [SimdVec3F],

    /* per-node-beam */
    pub node_face_node_index: &'backer [SimdU32],
    pub node_face_face_index: &'backer [SimdU32],
    pub node_face_vertex_index_for_node_0: &'backer [SimdU32],
    pub node_face_vertex_index_for_node_1: &'backer [SimdU32],
    pub node_face_vertex_index_for_node_2: &'backer [SimdU32],
}

const TAU: f32 = 6.283185307179586476925286766559;
const TOL: f32 = 0.0000001;
pub fn calculate_node_face_forces<'a>(
    inputs: &'a PerNodeFaceInput<'a>,
) -> impl ExactSizeIterator<Item = (SimdVec3F, SimdF32)> + use<'a> {
    let tol = simba::simd::Simd(SimdF32::splat(TOL));

    itertools::izip!(
        inputs.node_face_node_index,
        inputs.node_face_face_index,
        inputs.node_face_vertex_index_for_node_0,
        inputs.node_face_vertex_index_for_node_1,
        inputs.node_face_vertex_index_for_node_2
    ).map(move |(
        node_indices,
        face_indices,
        vertex_index_for_node_0,
        vertex_index_for_node_1,
        vertex_index_for_node_2
    )| {
        let select = |mask: SimdMask, true_values: nalgebra::Vector3<SimdF32>, false_values: nalgebra::Vector3<SimdF32>| -> nalgebra::Vector3<SimdF32> {
            unimplemented!()
        }; 

        let this_positions = position::get_positions_for_indices(&inputs.node_positions_unchanging, &inputs.node_positions_offset, *node_indices);
        let face_vertices = super::gather::gather_vec3([&inputs.face_node_indices], *face_indices)[0];

        // We have the node indices, so we don't need to do that weird magic
        let a = position::get_positions_for_indices(&inputs.node_positions_unchanging, &inputs.node_positions_offset, face_vertices[0]);
        let b = position::get_positions_for_indices(&inputs.node_positions_unchanging, &inputs.node_positions_offset, face_vertices[1]);
        let c  = position::get_positions_for_indices(&inputs.node_positions_unchanging, &inputs.node_positions_offset, face_vertices[2]);

        let ab = b-a;
        let ac = c-a;
        let bc = c-b;

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
            ac.dot(&bc)
        ];

        let [normal, nominal_angles] = super::gather::gather_vec3f([&inputs.face_normals, &inputs.face_nominal_angles], *face_indices);

        let angles_diff = algebrize(nominal_angles) - nalgebra::Vector3::new(angles[0], angles[1], angles[2]);

        // TODO: Calc forces
        // Line 178 and forward in velocity_calc.frag.glsl
    });
}