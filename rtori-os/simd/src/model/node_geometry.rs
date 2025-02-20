use super::*;

#[derive(Debug, Clone, Copy)]
pub struct NodeGeometryRange<const L: usize>
where
    LaneCount<L>: SupportedLaneCount,
{
    pub offset: SimdU32<L>,
    pub count: SimdU32<L>,
}

#[derive(Debug, Clone, Copy)]
pub struct NodeGeometry<const L: usize>
where
    LaneCount<L>: SupportedLaneCount,
{
    pub creases: NodeGeometryRange<L>,
    pub beams: NodeGeometryRange<L>,
    pub faces: NodeGeometryRange<L>,
}

impl<const L: usize> NodeGeometry<L>
where
    LaneCount<L>: SupportedLaneCount,
{
    const fn base_offsets_in_lanes() -> [usize; 6] {
        const SCALAR_SIZE_IN_BYTES: usize = core::mem::size_of::<u32>();

        let offset_offset = core::mem::offset_of!(NodeGeometryRange<L>, offset);
        let offset_count = core::mem::offset_of!(NodeGeometryRange<L>, count);

        [
            (core::mem::offset_of!(Self, creases) + offset_offset) / SCALAR_SIZE_IN_BYTES,
            (core::mem::offset_of!(Self, creases) + offset_count) / SCALAR_SIZE_IN_BYTES,
            (core::mem::offset_of!(Self, beams) + offset_offset) / SCALAR_SIZE_IN_BYTES,
            (core::mem::offset_of!(Self, beams) + offset_count) / SCALAR_SIZE_IN_BYTES,
            (core::mem::offset_of!(Self, faces) + offset_offset) / SCALAR_SIZE_IN_BYTES,
            (core::mem::offset_of!(Self, faces) + offset_count) / SCALAR_SIZE_IN_BYTES,
        ]
    }

    super::aosoa::impl_aosoa!(u32, L, 6);

    const fn from_array(arr: [SimdU32<L>; 6]) -> Self {
        Self {
            creases: NodeGeometryRange {
                offset: arr[0],
                count: arr[1],
            },
            beams: NodeGeometryRange {
                offset: arr[0],
                count: arr[1],
            },
            faces: NodeGeometryRange {
                offset: arr[0],
                count: arr[1],
            },
        }
    }
}
