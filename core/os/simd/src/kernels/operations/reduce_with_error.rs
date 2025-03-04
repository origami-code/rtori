use core::simd::cmp::SimdPartialOrd;
use core::simd::{LaneCount, SupportedLaneCount};

use crate::kernels::operations::gather::gather_vec3f_1;
use simd_common::{convert_nalgebra::to_nalgebra3, *};

pub struct ReduceWithErrorInput<'backer, const L: usize>
where
    LaneCount<L>: SupportedLaneCount,
{
    // Per node
    pub offset: &'backer [SimdU32<L>],
    pub count: &'backer [SimdU32<L>],

    // Per node-crease
    pub force: &'backer [SimdVec3F<L>],
    pub error: &'backer [SimdF32<L>],
}

pub fn reduce_with_error<'a, const L: usize>(
    inputs: &'a ReduceWithErrorInput<'a, L>,
) -> impl ExactSizeIterator<Item = (SimdVec3F<L>, SimdF32<L>)> + use<'a, L>
where
    LaneCount<L>: SupportedLaneCount,
    simba::simd::Simd<core::simd::Simd<f32, L>>: num_traits::Num + num_traits::NumAssign,
{
    let zero = simba::simd::Simd(SimdF32::splat(0.0));
    let zero_force = nalgebra::Vector3::new(zero, zero, zero);

    itertools::izip!(inputs.offset, inputs.count).map(move |(offset, count)| {
        let mut cursor = SimdU32::splat(0);

        let mut force = zero_force;
        let mut error = zero.0;

        loop {
            let mask = cursor.simd_lt(*count);

            // We execute while at least one of the lanes hasn't got the end
            if !mask.any() {
                break;
            }

            let current_offset = offset + cursor;
            let current_force = to_nalgebra3(gather_vec3f_1(inputs.force, current_offset));
            let current_error =
                simd_common::gather::gather_scalar_multi([&inputs.error], current_offset)[0];

            force += super::select(mask, current_force, zero_force);
            error += mask.select(current_error, zero.0);

            cursor += SimdU32::splat(1);
        }

        ([force.x.0, force.y.0, force.z.0], error)
    })
}
