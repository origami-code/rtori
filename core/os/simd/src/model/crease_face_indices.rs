use core::simd::{LaneCount, SupportedLaneCount};
use simd_common::{aosoa::impl_aosoa, SimdU32};

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
#[repr(transparent)]
pub struct CreaseFaceIndices<const L: usize>(pub [SimdU32<L>; 2])
where
    LaneCount<L>: SupportedLaneCount;

impl<const L: usize> CreaseFaceIndices<L>
where
    LaneCount<L>: SupportedLaneCount,
{
    const fn base_offsets_in_lanes() -> [usize; 2] {
        [0, L]
    }

    impl_aosoa!(u32, L, 2);

    const fn from_array(arr: [SimdU32<L>; 2]) -> Self {
        Self(arr)
    }
}
