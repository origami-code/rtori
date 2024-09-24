use rtori_os_model::NodeGeometry;

use crate::simd_atoms::*;

pub struct Nodes<'backer> {
    position_offset: &'backer mut [SimdVec3F],
    positions_unchanging: &'backer [SimdVec3F],
    normals: &'backer mut [SimdVec3F],
    external_forces: &'backer [SimdVec3F],
    mass: &'backer [SimdF32],
    fixed: &'backer [SimdMask],
    geometry: &'backer [NodeGeometry],
    velocity: &'backer mut [SimdVec3F],
}

pub struct CreaseGeometryLens {
    pub face_indices: [SimdU32; 2],
    pub complement_node_indices: [SimdU32; 2],
    pub adjacent_node_indices: [SimdU32; 2],
}

pub struct CreaseGeometries<'backer> {
    face_indices: [&'backer [SimdU32]; 2],
    complement_node_indices: [&'backer [SimdU32]; 2],
    adjacent_node_indices: [&'backer [SimdU32]; 2],
}

pub struct CreasesParameters<'backer> {
    geometry: &'backer CreaseGeometries<'backer>,
    k: &'backer [SimdF32],
    d: &'backer [SimdF32],
    target_fold_angle: &'backer [SimdF32],
}

pub struct CreasesFoldAngles<'backer> {
    fold_angles: &'backer mut [SimdF32],
}

#[derive(Debug, Clone, Copy)]
pub struct CreasesPhysicsLens {
    pub a_height: SimdF32,
    pub a_coef: SimdF32,

    pub b_height: SimdF32,
    pub b_coef: SimdF32,
}

impl CreasesPhysicsLens {
    #[inline]
    pub fn invalid() -> Self {
        let neg = SimdF32::splat(-1.0);

        Self {
            a_height: neg,
            a_coef: neg,
            b_height: neg,
            b_coef: neg,
        }
    }

    #[inline]
    pub fn simd_is_invalid(&self) -> SimdMask {
        use core::simd::cmp::SimdPartialOrd;

        let zero = SimdF32::splat(0.0);

        self
            .a_height
            .simd_le(zero)
    }


}

pub struct CreasePhysics<'backer> {
    pub a_height: &'backer [SimdF32],
    pub a_coef: &'backer [SimdF32],

    pub b_height: &'backer [SimdF32],
    pub b_coef: &'backer [SimdF32],
}

pub struct Faces<'backer> {
    indices_a: &'backer [SimdU32],
    indices_b: &'backer [SimdU32],
    nominal_angles: &'backer [SimdVec3F],
}


pub struct NodeCreases<'backer> {
    crease_indices: &'backer [SimdU32],
    node_number: &'backer [SimdU32],

    // scratch
    forces: &'backer mut [SimdVec3F],
    error: &'backer mut [SimdF32],
}

pub struct NodeBeams<'backer> {
    node_index: &'backer [SimdU32],
    k: &'backer [SimdF32],
    d: &'backer [SimdF32],
    length: &'backer [SimdF32],
    neighbour_index: &'backer [SimdU32],

    forces: &'backer mut [SimdVec3F],
}

pub struct NodeFaces<'backer> {
    node_index: &'backer [SimdU32],
    face_index: &'backer [SimdU32],

    forces: &'backer mut [SimdVec3F],
    error: &'backer mut [SimdF32],
}

pub struct State<'backer> {
    /* Configs */
    nodes: Nodes<'backer>,
    creases: CreasesParameters<'backer>,
    faces: Faces<'backer>,
    node_creases: NodeCreases<'backer>,
    node_beams: NodeBeams<'backer>,
    node_faces: NodeFaces<'backer>,
}
