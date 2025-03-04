use core::simd::{LaneCount, SupportedLaneCount};
use nalgebra as na;

#[inline(always)]
pub fn to_nalgebra3<const L: usize>(
    input: [core::simd::Simd<f32, L>; 3],
) -> na::Vector3<simba::simd::Simd<core::simd::Simd<f32, L>>>
where
    LaneCount<L>: SupportedLaneCount,
{
    na::Vector3::<simba::simd::Simd<core::simd::Simd<f32, L>>>::new(
        simba::simd::Simd(input[0]),
        simba::simd::Simd(input[1]),
        simba::simd::Simd(input[2]),
    )
}
