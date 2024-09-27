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
    let index_to_vector_start = indices * SimdU32::splat(3 * chunk_size);

    let x_indices = index_to_vector_start + SimdU32::splat(0 * chunk_size);
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
    gather_vec3f([input], indices)[0]
}

#[inline]
pub fn gather_vec3f<const N: usize, const L: usize>(
    slices: [&[SimdVec3F<L>]; N],
    indices: SimdU32<L>,
) -> [SimdVec3F<L>; N]
where
    LaneCount<L>: SupportedLaneCount,
{
    gather_vec3(slices, indices)
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
