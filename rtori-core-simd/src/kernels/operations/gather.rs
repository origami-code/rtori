use crate::simd_atoms::{SimdF32, SimdU32, SimdVec3F, SimdVec3U, CHUNK_SIZE};
use core::simd::{num::SimdUint, SimdElement};

#[inline]
pub fn indices_to_vec_as_scalar_indices(indices: crate::simd_atoms::SimdU32) -> SimdVec3U {
    // We load X, Y, Z from that
    let index_to_vector_start: SimdU32 =
        indices * SimdU32::splat(3 * u32::try_from(CHUNK_SIZE).unwrap());

    let x_indices: SimdU32 =
        index_to_vector_start + SimdU32::splat(0 * u32::try_from(CHUNK_SIZE).unwrap());
    let y_indices: SimdU32 =
        index_to_vector_start + SimdU32::splat(1 * u32::try_from(CHUNK_SIZE).unwrap());
    let z_indices: SimdU32 =
        index_to_vector_start + SimdU32::splat(2 * u32::try_from(CHUNK_SIZE).unwrap());

    [x_indices, y_indices, z_indices]
}

#[inline]
pub fn gather_vec3<T: SimdElement + bytemuck::Pod + Default, const N: usize>(
    slices: [&[[core::simd::Simd<T, {crate::simd_atoms::CHUNK_SIZE}>; 3]]; N],
    indices: crate::simd_atoms::SimdU32
) -> [[core::simd::Simd<T, {crate::simd_atoms::CHUNK_SIZE}>; 3]; N] {
    let [x_indices, y_indices, z_indices] = indices_to_vec_as_scalar_indices(indices);

    slices.map(|origin| {
        let scalars = bytemuck::cast_slice::<[core::simd::Simd<T, {crate::simd_atoms::CHUNK_SIZE}>; 3], T>(origin);

        let x = core::simd::Simd::gather_or_default(scalars, x_indices.cast::<usize>());
        let y = core::simd::Simd::gather_or_default(scalars, y_indices.cast::<usize>());
        let z = core::simd::Simd::gather_or_default(scalars, z_indices.cast::<usize>());

        [x, y, z]
    })
}

#[inline]
pub fn gather_vec3f_1(input: &[SimdVec3F], indices: crate::simd_atoms::SimdU32) -> SimdVec3F {
    gather_vec3f([input], indices)[0]
}

#[inline]
pub fn gather_vec3f<const N: usize>(
    slices: [&[SimdVec3F]; N],
    indices: crate::simd_atoms::SimdU32,
) -> [SimdVec3F; N] {
    gather_vec3(slices, indices)
}


#[inline]
pub fn gather_scalar<T: SimdElement + bytemuck::Pod + Default, const N: usize>(
    slices: [&[core::simd::Simd<T, {crate::simd_atoms::CHUNK_SIZE}>]; N],
    indices: crate::simd_atoms::SimdU32
) -> [core::simd::Simd<T, {crate::simd_atoms::CHUNK_SIZE}>; N] {
    slices.map(|slice| {
        let flattened = bytemuck::cast_slice::<core::simd::Simd<T, {crate::simd_atoms::CHUNK_SIZE}>, T>(slice);
        let gathered = core::simd::Simd::gather_or_default(flattened, indices.cast::<usize>());
        gathered
    })
}


#[inline]
pub fn gather_f32<const N: usize>(
    slices: [&[SimdF32]; N],
    indices: crate::simd_atoms::SimdU32
) -> [SimdF32; N] {
    gather_scalar(slices, indices)
}