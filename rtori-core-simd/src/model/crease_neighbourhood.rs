use super::*;

/// Always used together
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct CreaseNeighbourhood<const L: usize>
where
    LaneCount<L>: SupportedLaneCount,
{
    pub complement_node_indices: [SimdU32N<L>; 2],
    pub adjacent_node_indices: [SimdU32N<L>; 2],
}
