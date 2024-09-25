use core::simd::{LaneCount, Mask, SupportedLaneCount};

pub struct InputIteratorItem<T, const L: usize>
where
    LaneCount<L>: SupportedLaneCount,
{
    pub lens: T,
    pub mask: Mask<i32, L>,
}
