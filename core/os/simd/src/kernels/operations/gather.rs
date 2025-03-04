use core::simd::{LaneCount, SupportedLaneCount};
use simd_common::{gather::*, SimdF32, SimdU32, SimdVec3F};

#[inline]
pub fn gather_vec3f<const N: usize, const L: usize>(
    slices: [&[SimdVec3F<L>]; N],
    indices: SimdU32<L>,
) -> [SimdVec3F<L>; N]
where
    LaneCount<L>: SupportedLaneCount,
{
    gather_vec3_multi(slices, indices)
}

#[inline]
pub fn gather_vec3f_1<const L: usize>(
    input: &[SimdVec3F<L>],
    indices: simd_common::SimdU32<L>,
) -> SimdVec3F<L>
where
    LaneCount<L>: SupportedLaneCount,
{
    gather_vec3(input, indices)
}

#[inline]
pub fn gather_f32<const N: usize, const L: usize>(
    slices: [&[SimdF32<L>]; N],
    indices: SimdU32<L>,
) -> [SimdF32<L>; N]
where
    LaneCount<L>: SupportedLaneCount,
{
    gather_scalar_multi(slices, indices)
}

#[cfg(test)]
mod tests {
    use super::*;
    const SAMPLE_POSITIONS_SIMD4: [[[f32; 4]; 3]; 2] = [
        [
            /*      0      1     2    3 */
            /*x*/ [0.0, 0.0, 0.0, 1.0],
            /*y*/ [1.0, 0.0, -1.0, 0.0],
            /*z*/ [0.0, 1.0, 0.0, 0.0],
        ],
        [
            /*      4      5     6    7 */
            /*x*/ [0.0, 0.0, 0.0, 0.0],
            /*y*/ [0.0, 0.0, 0.0, 0.0],
            /*z*/ [-1.0, -1.0, 0.0, 0.0],
        ],
    ];

    const SAMPLE_POSITIONS_SCALAR: [[f32; 3]; 6] = [
        [0.0, 1.0, 0.0],
        [0.0, 0.0, 1.0],
        [0.0, -1.0, 0.0],
        [1.0, 0.0, 0.0],
        [0.0, 0.0, -1.0],
        [0.0, 0.0, -1.0],
    ];

    #[test]
    pub fn test_gather_vec3f_1() {
        let sample_positions_simd4 =
            SAMPLE_POSITIONS_SIMD4.map(|arr| arr.map(|inner| SimdF32::from_array(inner)));

        for (i, expect) in SAMPLE_POSITIONS_SCALAR.iter().enumerate() {
            let indices = SimdU32::splat(i as u32);
            let actual = gather_vec3f_1(&sample_positions_simd4, indices);

            for j in 0..=3 {
                let x = actual[0][j];
                let y = actual[1][j];
                let z = actual[2][j];

                assert_eq!([x, y, z], *expect, "index request {i} not matching");
            }
        }
    }
}
