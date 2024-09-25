use super::position;
use crate::simd_atoms::*;
use nalgebra as na;

pub struct PerFaceInputs<'backer> {
    pub face_node_indices: &'backer [SimdVec3U],
    pub face_count: usize,

    pub node_positions_offset: &'backer [SimdVec3F],
    pub node_positions_unchanging: &'backer [SimdVec3F],
}

pub struct PerFaceOutputs<'backer> {
    face_normals: &'backer mut [SimdVec3F],
}

pub fn calculate_normals<'a>(
    inputs: &'a PerFaceInputs<'a>,
) -> impl ExactSizeIterator<Item = na::Vector3<simba::simd::Simd<SimdF32>>> + use<'a> {
    // First, we calculate every combined position using SIMD
    inputs.face_node_indices.iter().map(move |idx: &SimdVec3U| {
        let f = #[inline(always)]
        |indices: SimdU32| {
            position::get_positions_for_indices(
                inputs.node_positions_unchanging,
                inputs.node_positions_offset,
                indices,
            )
        };

        // Gather
        let a = f(idx[0]);
        let b = f(idx[1]);
        let c = f(idx[2]);

        let ba = b - a;
        let ca = c - a;

        let result = ba.cross(&ca).normalize();
        result
    })
}

// TODO: tests
