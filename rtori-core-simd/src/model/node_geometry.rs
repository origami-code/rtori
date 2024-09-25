use super::*;

#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct NodeGeometryRange<const L: usize>
where
LaneCount<L>: SupportedLaneCount
{
    pub offset: SimdU32N<L>,
    pub count: SimdU32N<L>
}

#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct NodeGeometry<const L: usize>
where
    LaneCount<L>: SupportedLaneCount,
{
    pub creases: NodeGeometryRange<L>,
    pub beams: NodeGeometryRange<L>,
    pub faces: NodeGeometryRange<L>
}