use aosoa::impl_aosoa;

use super::*;

/// Always used together
#[derive(Debug, Clone, Copy)]
pub struct CreaseNeighbourhood<const L: usize>
where
    LaneCount<L>: SupportedLaneCount,
{
    pub complement_node_indices: [SimdU32<L>; 2],
    pub adjacent_node_indices: [SimdU32<L>; 2],
}

impl<const L: usize> CreaseNeighbourhood<L>
where
    LaneCount<L>: SupportedLaneCount,
{
    const fn base_offsets_in_lanes() -> [usize; 4] {
        const SCALAR_SIZE_IN_BYTES: usize = core::mem::size_of::<u32>();

        [
            core::mem::offset_of!(Self, complement_node_indices) / SCALAR_SIZE_IN_BYTES,
            (core::mem::offset_of!(Self, complement_node_indices) + Self::SIMD_SIZE)
                / SCALAR_SIZE_IN_BYTES,
            core::mem::offset_of!(Self, adjacent_node_indices) / SCALAR_SIZE_IN_BYTES,
            (core::mem::offset_of!(Self, adjacent_node_indices) + Self::SIMD_SIZE)
                / SCALAR_SIZE_IN_BYTES,
        ]
    }

    super::aosoa::impl_aosoa!(u32, L, 4);

    const fn from_array(arr: [SimdU32<L>; 4]) -> Self {
        Self {
            complement_node_indices: [arr[0], arr[1]],
            adjacent_node_indices: [arr[1], arr[2]],
        }
    }
}
