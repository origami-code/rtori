use crate::*;
use core::simd::{LaneCount, SupportedLaneCount};

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
