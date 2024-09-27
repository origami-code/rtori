use core::simd::{LaneCount, SupportedLaneCount};

use crate::simd_atoms::*;

#[inline]
pub fn select<const L: usize>(
    mask: core::simd::Mask<i32, L>,
    true_values: nalgebra::Vector3<simba::simd::Simd<SimdF32<L>>>,
    false_values: nalgebra::Vector3<simba::simd::Simd<SimdF32<L>>>,
) -> nalgebra::Vector3<simba::simd::Simd<SimdF32<L>>>
where
    LaneCount<L>: SupportedLaneCount,
{
    let x = mask.select(true_values.x.0, false_values.x.0);

    let y = mask.select(true_values.y.0, false_values.y.0);

    let z = mask.select(true_values.z.0, false_values.z.0);

    nalgebra::Vector3::new(
        simba::simd::Simd(x),
        simba::simd::Simd(y),
        simba::simd::Simd(z),
    )
}
