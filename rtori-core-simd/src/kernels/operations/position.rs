use core::simd::{LaneCount, SupportedLaneCount};

use super::algebra::algebrize;
use super::gather::gather_vec3f;

use crate::simd_atoms::*;

use nalgebra as na;

#[inline]
pub fn get_positions_for_indices<const L: usize>(
    positions_unchanging: &[SimdVec3F<L>],
    positions_offsets: &[SimdVec3F<L>],
    indices: SimdU32<L>,
) -> na::Vector3<simba::simd::Simd<SimdF32<L>>>
where
    LaneCount<L>: SupportedLaneCount,
    simba::simd::Simd<core::simd::Simd<f32, L>>: num_traits::Num + num_traits::NumAssign,
{
    let [unchanging, offset] = gather_vec3f([positions_unchanging, positions_offsets], indices);

    let unchanging = algebrize(unchanging);
    let offset = algebrize(offset);

    unchanging + offset
}
