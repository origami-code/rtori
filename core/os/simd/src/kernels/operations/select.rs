use core::simd::{LaneCount, SupportedLaneCount};

use simd_common::*;

#[inline]
pub fn select_n<const L: usize, const N: usize>(
    mask: core::simd::Mask<i32, L>,
    true_values: [SimdF32<L>; N],
    false_values: [SimdF32<L>; N],
) -> [SimdF32<L>; N]
where
    LaneCount<L>: SupportedLaneCount,
{
    let mut output = [SimdF32::splat(0.0); N];
    for i in 0..N {
        output[i] = mask.select(true_values[i], false_values[i]);
    }
    output
}

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
