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
    simba::simd::Simd<core::simd::Simd<f32, L>>:
        simba::simd::SimdComplexField<SimdRealField = simba::simd::Simd<core::simd::Simd<f32, L>>>,
{
    // First, we calculate every combined position using SIMD
    inputs
        .face_node_indices
        .iter()
        .map(move |idx: &SimdVec3U<L>| {
            // /*2024-10-11*/ println!("PerFaceInput: processing faces {:?}", idx);
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

            let ab = b - a;
            let ac = c - a;
            let cross = ab.cross(&ac);

            let norm = {
                let norm: simba::simd::Simd<SimdF32<L>> = cross.norm();
                use std::simd::cmp::SimdPartialEq as _;
                let norm_is_zero = norm.0.simd_eq(SimdF32::splat(0.0));
                let norm_corrected = norm_is_zero.select(SimdF32::splat(1.0), norm.0);
                simba::simd::Simd(norm_corrected)
            };

            let result = cross.unscale(norm);
            /* 2025-01-13 */ /*println!(
                "Normals are: {:?} (ab: {:?}, ac: {:?}, cross: {:?}, cross_norm: {:?})",
                result, ab, ac, cross, norm
            );*/
            PerFaceOutput {
                face_normals: [result.x.0, result.y.0, result.z.0],
            }
        })
}

// TODO: tests
