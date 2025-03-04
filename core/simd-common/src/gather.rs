use crate::SimdU32;
use core::simd::{num::SimdUint, LaneCount, SimdElement, SupportedLaneCount};

/// Converts a vector of logical indices (into vectors of dimension D) into D outer vectors of indices so as to access AoSoA layouts.
///
/// Given a in-memory layout of X0 X1 X2 X3 Y0 Y1 Y2 Y3 Z0 Z1 Z2 Z3 X4 X5 X6 X7 Y4 Y5 Y6 Y7 Z4 Z5 Z6 Z7
/// And an input of 0 1 4 5
/// This function returns the indices of X0 X1 X4 X5 Y0 Y1 Y4 Y5 Z0 Z1 Z4 Z5
/// So
/// x: 0 1 12 13
/// y: 4 5 16 17
/// z: 8 9 20 21
#[inline]
pub fn indices_to_vec_as_scalar_indices<const L: usize, const D: usize>(
    indices: core::simd::Simd<u32, L>,
) -> [core::simd::Simd<u32, L>; D]
where
    LaneCount<L>: SupportedLaneCount,
{
    let chunk_size = u32::try_from(L).unwrap();

    // We load X, Y, Z, ... from that
    let lanes_splat = SimdU32::splat(chunk_size);

    let vec_index = indices / lanes_splat;
    let inner_index = indices % lanes_splat;

    let index_to_vector_start =
        vec_index * SimdU32::splat(u32::try_from(D * L).unwrap()) + inner_index;

    let mut output = [SimdU32::splat(0); D];
    for i in 0..(D as u32) {
        output[i as usize] = index_to_vector_start + SimdU32::splat(i * chunk_size);
    }
    output
}

/// Gathers from a set of slices of vectors (of dimension 3) with the same vector of logical indices, to access AoSoA layouts.
#[inline]
pub fn gather_vec3_multi<T, const N: usize, const L: usize>(
    slices: [&[[core::simd::Simd<T, L>; 3]]; N],
    indices: SimdU32<L>,
) -> [[core::simd::Simd<T, L>; 3]; N]
where
    T: SimdElement + bytemuck::Pod + Default,
    LaneCount<L>: SupportedLaneCount,
{
    let [x_indices, y_indices, z_indices] = indices_to_vec_as_scalar_indices(indices);
    /*  /*2024-10-11*/ println println!(
        "Indices: x: {:?}, y: {:?}, z: {:?}",
        x_indices, y_indices, z_indices
    );*/
    slices.map(|origin| {
        let scalars = bytemuck::cast_slice::<[core::simd::Simd<T, L>; 3], T>(origin);

        let x = core::simd::Simd::gather_or_default(scalars, x_indices.cast::<usize>());
        let y = core::simd::Simd::gather_or_default(scalars, y_indices.cast::<usize>());
        let z = core::simd::Simd::gather_or_default(scalars, z_indices.cast::<usize>());

        [x, y, z]
    })
}

#[inline]
pub fn gather_vec3<T, const L: usize>(
    input: &[[core::simd::Simd<T, L>; 3]],
    indices: crate::SimdU32<L>,
) -> [core::simd::Simd<T, L>; 3]
where
    T: SimdElement + bytemuck::Pod + Default,
    LaneCount<L>: SupportedLaneCount,
{
    gather_vec3_multi([input], indices)[0]
}

/// Gathers from a set of slices of scalars with the same vector of logical indices, to access vectors in lockstep.
#[inline]
pub fn gather_scalar_multi<T, const N: usize, const L: usize>(
    slices: [&[core::simd::Simd<T, L>]; N],
    indices: SimdU32<L>,
) -> [core::simd::Simd<T, L>; N]
where
    T: SimdElement + bytemuck::Pod + Default,
    LaneCount<L>: SupportedLaneCount,
{
    slices.map(|slice| {
        let flattened = bytemuck::cast_slice::<core::simd::Simd<T, L>, T>(slice);
        core::simd::Simd::gather_or_default(flattened, indices.cast::<usize>())
    })
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
            SAMPLE_POSITIONS_SIMD4.map(|arr| arr.map(|inner| crate::SimdF32::from_array(inner)));

        for (i, expect) in SAMPLE_POSITIONS_SCALAR.iter().enumerate() {
            let indices = SimdU32::splat(i as u32);
            let actual = gather_vec3(&sample_positions_simd4, indices);

            for j in 0..=3 {
                let x = actual[0][j];
                let y = actual[1][j];
                let z = actual[2][j];

                assert_eq!([x, y, z], *expect, "index request {i} not matching");
            }
        }
    }
}
