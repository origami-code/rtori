use core::simd::{LaneCount, SupportedLaneCount};
use simd_common::{impl_aosoa_flat, SimdU32};

/// Always used together
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct NodeCreasePointer<const L: usize>
where
    LaneCount<L>: SupportedLaneCount,
{
    pub crease_indices: SimdU32<L>,
    pub node_number: SimdU32<L>,
}

impl<const L: usize> NodeCreasePointer<L>
where
    LaneCount<L>: SupportedLaneCount,
{
    const fn from_array(arr: [SimdU32<L>; 2]) -> Self {
        Self {
            crease_indices: arr[0],
            node_number: arr[1],
        }
    }

    impl_aosoa_flat!(u32, L, 2, [crease_indices, node_number]);
}
