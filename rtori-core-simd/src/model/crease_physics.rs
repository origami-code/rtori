use super::*;

/// Always used together
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct CreasesPhysicsLens<const L: usize>
where
    LaneCount<L>: SupportedLaneCount,
{
    pub a_height: SimdF32<L>,
    pub a_coef: SimdF32<L>,

    pub b_height: SimdF32<L>,
    pub b_coef: SimdF32<L>,
}

impl<const L: usize> CreasesPhysicsLens<L>
where
    LaneCount<L>: SupportedLaneCount,
{
    #[inline]
    pub fn invalid() -> Self {
        let neg = SimdF32::splat(-1.0);

        Self {
            a_height: neg,
            a_coef: neg,
            b_height: neg,
            b_coef: neg,
        }
    }

    #[inline]
    pub fn simd_is_invalid(&self) -> SimdMask<L> {
        use core::simd::cmp::SimdPartialOrd;

        let zero = SimdF32::splat(0.0);

        self.a_height.simd_le(zero)
    }
}

impl<const L: usize> CreasesPhysicsLens<L>
where
    LaneCount<L>: SupportedLaneCount,
{
    super::aosoa::impl_aosoa_flat!(f32, L, 4, [a_height, a_coef, b_height, b_coef]);

    const fn from_array(arr: [SimdF32<L>; 4]) -> Self {
        Self {
            a_height: arr[0],
            a_coef: arr[1],
            b_height: arr[2],
            b_coef: arr[3],
        }
    }
}

#[cfg(test)]
mod test {
    use core::simd::{LaneCount, SupportedLaneCount};

    use super::CreasesPhysicsLens;
    use crate::simd_atoms::{SimdF32, SimdU32};

    pub fn test_uniform<const L: usize>()
    where
        LaneCount<L>: SupportedLaneCount,
    {
        let a_height = SimdF32::<{ L }>::splat(0.0);
        let a_coef = SimdF32::<{ L }>::splat(1.0);

        let b_height = SimdF32::<{ L }>::splat(2.0);
        let b_coef = SimdF32::<{ L }>::splat(3.0);

        let source = [
            CreasesPhysicsLens {
                a_height,
                a_coef,
                b_height,
                b_coef,
            },
            CreasesPhysicsLens {
                a_height,
                a_coef,
                b_height,
                b_coef,
            },
        ];

        for i in 0..L {
            let mut indices = [0; L];
            for k in 0..L {
                indices[k] = k + i;
            }
            let indices = SimdU32::<{ L }>::from(indices.map(|idx| u32::try_from(idx).unwrap()));

            let result = CreasesPhysicsLens::gather(&source, indices);

            assert_eq!(
                a_height, result.a_height,
                "indices {:?}: a_height should match",
                indices
            );
            assert_eq!(
                a_coef, result.a_coef,
                "indices {:?}: a_coef should match",
                indices
            );
            assert_eq!(
                b_height, result.b_height,
                "indices {:?}: b_height should match",
                indices
            );
            assert_eq!(
                b_coef, result.b_coef,
                "indices {:?}: b_coef should match",
                indices
            );
        }
    }

    #[test]
    pub fn test_uniform_l1() {
        test_uniform::<1>();
    }

    #[test]
    pub fn test_uniform_l2() {
        test_uniform::<2>();
    }

    #[test]
    pub fn test_uniform_l4() {
        test_uniform::<4>();
    }

    #[test]
    pub fn test_uniform_l8() {
        test_uniform::<8>();
    }

    #[test]
    pub fn test_uniform_l16() {
        test_uniform::<16>();
    }
}
