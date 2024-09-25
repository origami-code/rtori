use super::*;

/// Always used together
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
#[repr(C)]
pub struct CreasesPhysicsLens<const L: usize>
where
    LaneCount<L>: SupportedLaneCount,
{
    pub a_height: SimdF32N<L>,
    pub a_coef: SimdF32N<L>,

    pub b_height: SimdF32N<L>,
    pub b_coef: SimdF32N<L>,
}

impl<const L: usize> CreasesPhysicsLens<L>
where
    LaneCount<L>: SupportedLaneCount,
{
    #[inline]
    pub fn invalid() -> Self {
        let neg = SimdF32N::splat(-1.0);

        Self {
            a_height: neg,
            a_coef: neg,
            b_height: neg,
            b_coef: neg,
        }
    }

    #[inline]
    pub fn simd_is_invalid(&self) -> SimdMaskN<L> {
        use core::simd::cmp::SimdPartialOrd;

        let zero = SimdF32N::splat(0.0);

        self.a_height.simd_le(zero)
    }
}
