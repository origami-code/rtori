use super::algebra::algebrize;
use super::gather::gather_vec3f;

use crate::simd_atoms::*;

use nalgebra as na;

#[inline]
pub fn get_positions_for_indices(
    positions_unchanging: &[SimdVec3F],
    positions_offsets: &[SimdVec3F],
    indices: SimdU32,
) -> na::Vector3<simba::simd::Simd<SimdF32>> {
    let [unchanging, offset] = gather_vec3f([positions_unchanging, positions_offsets], indices);

    let unchanging = algebrize(unchanging);
    let offset = algebrize(offset);

    unchanging + offset
}
