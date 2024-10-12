use crate::simd_atoms::{SimdF32, SimdU32, SimdVec3F};
use core::simd::{num::SimdUint, LaneCount, SimdElement, SupportedLaneCount};

#[inline]
pub fn indices_to_vec_as_scalar_indices<const L: usize>(
    indices: core::simd::Simd<u32, L>,
) -> [core::simd::Simd<u32, L>; 3]
where
    LaneCount<L>: SupportedLaneCount,
{
    let chunk_size = u32::try_from(L).unwrap();

    // We load X, Y, Z from that
    let lanes_splat = SimdU32::splat(u32::try_from(chunk_size).unwrap());

    let vec_index = indices / lanes_splat;
    let inner_index = indices % lanes_splat;

    let index_to_vector_start =
        vec_index * SimdU32::splat(u32::try_from(3 * L).unwrap()) + inner_index;

    let x_indices = index_to_vector_start; //+ SimdU32::splat(0 * chunk_size);
    let y_indices = index_to_vector_start + SimdU32::splat(1 * chunk_size);
    let z_indices = index_to_vector_start + SimdU32::splat(2 * chunk_size);

    [x_indices, y_indices, z_indices]
}

#[inline]
pub fn gather_vec3<T: SimdElement + bytemuck::Pod + Default, const N: usize, const L: usize>(
    slices: [&[[core::simd::Simd<T, L>; 3]]; N],
    indices: SimdU32<L>,
) -> [[core::simd::Simd<T, L>; 3]; N]
where
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
pub fn gather_vec3f_1<const L: usize>(
    input: &[SimdVec3F<L>],
    indices: crate::simd_atoms::SimdU32<L>,
) -> SimdVec3F<L>
where
    LaneCount<L>: SupportedLaneCount,
{
    let result = gather_vec3f([input], indices)[0];
    /* /*2024-10-11*/ println!(
        "gather_vec3f_1: indices: {:?} => result: {:?} [input {:?}]",
        indices, result, input
    ); */
    result
}

#[inline]
pub fn gather_vec3f<const N: usize, const L: usize>(
    slices: [&[SimdVec3F<L>]; N],
    indices: SimdU32<L>,
) -> [SimdVec3F<L>; N]
where
    LaneCount<L>: SupportedLaneCount,
{
    let result = gather_vec3(slices, indices);

    /* /*2024-10-11*/ println!(
        "gather_vec3f: indices: {:?} => result: {:?} [input {:?}]",
        indices, result, slices
    ); */
    result
}

#[inline]
pub fn gather_scalar<T: SimdElement + bytemuck::Pod + Default, const N: usize, const L: usize>(
    slices: [&[core::simd::Simd<T, L>]; N],
    indices: SimdU32<L>,
) -> [core::simd::Simd<T, L>; N]
where
    LaneCount<L>: SupportedLaneCount,
{
    slices.map(|slice| {
        let flattened = bytemuck::cast_slice::<core::simd::Simd<T, L>, T>(slice);
        let gathered = core::simd::Simd::gather_or_default(flattened, indices.cast::<usize>());
        gathered
    })
}

#[inline]
pub fn gather_f32<const N: usize, const L: usize>(
    slices: [&[SimdF32<L>]; N],
    indices: SimdU32<L>,
) -> [SimdF32<L>; N]
where
    LaneCount<L>: SupportedLaneCount,
{
    gather_scalar(slices, indices)
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
