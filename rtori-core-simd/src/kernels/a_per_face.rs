use core::simd::{LaneCount, SupportedLaneCount};

use super::position;
use crate::simd_atoms::*;
use nalgebra as na;

#[derive(Debug)]
pub struct PerFaceInputs<'backer, const L: usize>
where
    LaneCount<L>: SupportedLaneCount,
{
    pub face_node_indices: &'backer [SimdVec3U<L>],

    pub node_positions_offset: &'backer [SimdVec3F<L>],
    pub node_positions_unchanging: &'backer [SimdVec3F<L>],
}

pub struct PerFaceOutput<const L: usize>
where
    LaneCount<L>: SupportedLaneCount,
{
    pub face_normals: SimdVec3F<L>,
}

#[inline]
pub fn calculate_normals<'a, const L: usize>(
    inputs: &'a PerFaceInputs<'a, L>,
) -> impl ExactSizeIterator<Item = PerFaceOutput<L>> + use<'a, L>
where
    LaneCount<L>: SupportedLaneCount,
    simba::simd::Simd<core::simd::Simd<f32, L>>: simba::simd::SimdComplexField,
{
    // First, we calculate every combined position using SIMD
    inputs
        .face_node_indices
        .iter()
        .map(move |idx: &SimdVec3U<L>| {
            let f = #[inline(always)]
            |indices: SimdU32<L>| {
                position::get_positions_for_indices(
                    inputs.node_positions_unchanging,
                    inputs.node_positions_offset,
                    indices,
                )
            };

            // Gather
            let [a, b, c] = idx.map(f);

            let ba = b - a;
            let ca = c - a;

            let result = ba.cross(&ca).normalize();
            PerFaceOutput {
                face_normals: [result.x.0, result.y.0, result.z.0],
            }
        })
}

// TODO: tests
