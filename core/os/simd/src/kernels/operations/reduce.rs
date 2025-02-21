use core::simd::cmp::SimdPartialOrd;
use core::simd::{LaneCount, SupportedLaneCount};

use crate::kernels::operations::gather::gather_vec3f_1;
use simd_common::{convert_nalgebra::to_nalgebra3, *};

pub struct ReduceInput<'backer, const L: usize>
where
    LaneCount<L>: SupportedLaneCount,
{
    // Per node
    pub offset: &'backer [SimdU32<L>],
    pub count: &'backer [SimdU32<L>],

    // Per node-crease
    pub force: &'backer [SimdVec3F<L>],
}

pub fn reduce<'a, const L: usize>(
    inputs: &'a ReduceInput<'a, L>,
) -> impl ExactSizeIterator<Item = SimdVec3F<L>> + use<'a, L>
where
    LaneCount<L>: SupportedLaneCount,
    simba::simd::Simd<core::simd::Simd<f32, L>>: num_traits::Num + num_traits::NumAssign,
{
    let zero = simba::simd::Simd(SimdF32::splat(0.0));
    let zero_force = nalgebra::Vector3::new(zero, zero, zero);

    itertools::izip!(inputs.offset, inputs.count).map(move |(offset, count)| {
        let mut cursor = SimdU32::splat(0);
        let mut force = zero_force;
        loop {
            let mask = cursor.simd_lt(*count);

            // We execute while at least one of the lanes hasn't got the end
            if !mask.any() {
                break;
            }

            let current_offset = offset + cursor;
            let current_force = to_nalgebra3(gather_vec3f_1(inputs.force, current_offset));

            force += super::select(mask, current_force, zero_force);

            cursor += SimdU32::splat(1);
        }

        [force.x.0, force.y.0, force.z.0]
    })
}
