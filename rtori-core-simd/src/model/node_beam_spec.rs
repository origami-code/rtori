use core::simd::num::SimdUint;

use super::*;
use aosoa::impl_aosoa_flat;

/// Always used together
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct NodeBeamSpec<const L: usize>
where
    LaneCount<L>: SupportedLaneCount,
{
    pub node_indices: SimdU32<L>,
    pub length: SimdF32<L>,
    pub neighbour_indices: SimdF32<L>,
}

impl<const L: usize> NodeBeamSpec<L>
where
    LaneCount<L>: SupportedLaneCount,
{
    super::aosoa::impl_aosoa_flat!(u32, L, 3, [node_indices, length, neighbour_indices]);

    fn from_array(arr: [SimdU32<L>; 3]) -> Self {
        Self {
            node_indices: arr[0],
            length: SimdF32::from_array(arr[1].as_array().map(|bits| f32::from_bits(bits))),
            neighbour_indices: SimdF32::from_array(
                arr[2].as_array().map(|bits| f32::from_bits(bits)),
            ),
        }
    }
}
