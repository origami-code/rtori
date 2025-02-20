use core::simd::{LaneCount, SimdElement, SupportedLaneCount};

use crate::simd_atoms::*;

pub(crate) fn gather_simd_field_single<T, const L: usize>(
    flattened: &[T],
    base: SimdU32<L>,
    offset_in_lanes: usize,
) -> core::simd::Simd<T, L>
where
    T: SimdElement + Default,
    LaneCount<L>: SupportedLaneCount,
{
    use core::simd::num::SimdUint as _;

    let offset = u32::try_from(offset_in_lanes).unwrap();
    let indices = base + SimdU32::splat(offset);
    let got = core::simd::Simd::gather_or_default(flattened, indices.cast::<usize>());
    got
}

// AoSoA (L = 2)
// A: Array
// C: Struct
// S: SIMD
// V: Value
// Layout:   |00|04|08|12|16|20|24|28|32|36|40|44|48|52|56|60|
//           |00|01|02|03|04|05|06|07|08|09|10|11|12|13|14|15|
//           |S--X-|S--X-|S--X-|S--X-|SY---|SY---|SY---|SY---|
//           |C                      |C                      |
// As such the math to select indices I = [1,2] would be:
// I mod L = [1, 0]
// I div L = [0, 1]
// base = (I div L) * SIMD_COUNT * L + (I rem L) = [1, 8]
// index_simd0 = base + ORDER_SIMD0 * L = [1, 8]
// index_simd1 = base + ORDER_SIMD1 * L = [3, 10]
// index_simd2 = base + ORDER_SIMD2 * L = [5, 12]
// index_simd3 = base + ORDER_SIMD3 * L = [7, 14]

/// For a complex AoSoA layout where SIMD type may be nested, even though in the end the struct is composed
/// only of SIMD types, with no paddings
macro_rules! impl_aosoa(
    ($scalar_type:ty, $lane_generic:ident, $field_count:expr) => {
        const STRUCT_SIZE: usize = core::mem::size_of::<Self>();
        const SIMD_SIZE: usize = core::mem::size_of::<core::simd::Simd<$scalar_type, $lane_generic>>();
        const SIMD_COUNT: usize = $field_count;

        #[inline]
        fn compute_base_offset(indices: SimdU32<$lane_generic>) -> SimdU32<$lane_generic> {
            let lanes_splat = SimdU32::splat(u32::try_from($lane_generic).unwrap());
            let struct_index = indices / lanes_splat;
            let inner_index = indices % lanes_splat;

            let base = struct_index * SimdU32::splat(u32::try_from(Self::SIMD_COUNT * $lane_generic).unwrap()) + inner_index;
            base
        }

        #[inline]
        fn gather_simd_field_single(flattened: &[$scalar_type], base: SimdU32<$lane_generic>, field_order: usize) -> core::simd::Simd<$scalar_type, $lane_generic> {
            let offsets = Self::base_offsets_in_lanes();
            $crate::model::aosoa::gather_simd_field_single(flattened, base, offsets[field_order])
        }

        #[inline]
        fn gather_simd_field_all(flattened: &[$scalar_type], base: SimdU32<$lane_generic>) -> [core::simd::Simd<$scalar_type, $lane_generic>; $field_count] {
            let get_from_offset = |order| Self::gather_simd_field_single(flattened, base, order);
            let mut field_orders = [0; $field_count];
            for i in 0..$field_count {
                field_orders[i] = i;
            }

            field_orders.map(|order| get_from_offset(order))
        }

        #[inline]
        pub fn gather(arr: &[Self], indices: SimdU32<$lane_generic>) -> Self {
            assert_eq!(Self::SIMD_SIZE, core::mem::size_of::<[$scalar_type; $lane_generic]>() );

            assert_eq!(
                    Self::SIMD_SIZE * $field_count,
                    Self::STRUCT_SIZE,
                    "The struct must be made up of ONLY SIMD types. No padding allowed (though they can be reordered)."
            );

            let flattened = {
                let (pre, cur, post) = unsafe { arr.align_to::<$scalar_type>() };
                assert!(pre.len() == 0 && post.len() == 0);
                cur
            };

            let base = Self::compute_base_offset(indices);

            let values = Self::gather_simd_field_all(flattened, base);
            Self::from_array(values)
        }
    }
);
pub(crate) use impl_aosoa;

/// For a simple AoSoA layout where every SIMD type is directly on the struct
macro_rules! impl_aosoa_flat(
    ($scalar_type:ty, $lane_generic:ident, $field_count:expr, [$($field:expr),+]) => {
        const fn base_offsets_in_lanes() -> [usize; $field_count] {
            const SCALAR_SIZE_IN_BYTES: usize = core::mem::size_of::<$scalar_type>();

            [
                $(
                    core::mem::offset_of!(Self, $field) / SCALAR_SIZE_IN_BYTES
                ),+
            ]
        }

        $crate::model::aosoa::impl_aosoa!($scalar_type, $lane_generic, $field_count);
    }
);
pub(crate) use impl_aosoa_flat;
