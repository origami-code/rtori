use super::*;
use aosoa::impl_aosoa_flat;

/// Always used together
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct NodeBeamSpec<const L: usize>
where
    LaneCount<L>: SupportedLaneCount,
{
    pub node_indices: SimdU32<L>,
    pub neighbour_indices: SimdU32<L>,
}

impl<const L: usize> NodeBeamSpec<L>
where
    LaneCount<L>: SupportedLaneCount,
{
    super::aosoa::impl_aosoa_flat!(u32, L, 2, [node_indices, neighbour_indices]);

    fn from_array(arr: [SimdU32<L>; 2]) -> Self {
        Self {
            node_indices: arr[0],
            neighbour_indices: arr[1],
        }
    }
}
