use super::*;
use aosoa::impl_aosoa_flat;

/// Always used together
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct NodeFaceSpec<const L: usize>
where
    LaneCount<L>: SupportedLaneCount,
{
    pub node_indices: SimdU32<L>,
    pub face_indices: SimdU32<L>,
}

impl<const L: usize> NodeFaceSpec<L>
where
    LaneCount<L>: SupportedLaneCount,
{
    super::aosoa::impl_aosoa_flat!(u32, L, 2, [node_indices, face_indices]);

    fn from_array(arr: [SimdU32<L>; 2]) -> Self {
        Self {
            node_indices: arr[0],
            face_indices: arr[1],
        }
    }
}
