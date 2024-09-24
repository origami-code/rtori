use crate::simd_atoms::{SimdF32, SimdVec3F};
use nalgebra as na;

#[inline(always)]
pub fn algebrize(input: SimdVec3F) -> na::Vector3<simba::simd::Simd<SimdF32>> {
    na::Vector3::<simba::simd::Simd<SimdF32>>::new(
        simba::simd::Simd(input[0]),
        simba::simd::Simd(input[1]),
        simba::simd::Simd(input[1]),
    )
}
