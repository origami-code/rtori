use crate::SimdU32;
use core::simd::{LaneCount, SimdElement, SupportedLaneCount};
pub use static_assertions;

#[doc(hidden)]
pub fn gather_simd_field_single<T, const L: usize>(
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
    core::simd::Simd::gather_or_default(flattened, indices.cast::<usize>())
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

/// Implements useful methods on an AoSoA struct, for a complex layout where SIMD types may be found
/// nested. Works as long though as the struct is composed only of SIMD types, with no paddings.
///
/// This method requires two methods to be defined:
/// - `base_offsets_in_lanes` specifying the offsets of each SIMD field in the struct, in number of lanes.
/// - `from_array` building the struct from an array of the base element
///
/// Thes methods as well as other ones may refer to convenience associated constants defined on the type:
///
/// - `Self::STRUCT_SIZE`: a shorthand for `core::mem::size_of::<Self>()`
/// - `Self::SIMD_SIZE`: the size in bytes of a simd element of the struct
/// - `Self::SIMD_COUNT`: the amount of these packed vectors that the struct contains
///
/// For example:
///
/// ```
/// #![feature(portable_simd)]
///
/// # use simd_common::aosoa;
///
/// pub struct ContainingStruct<const L: usize>
/// where
///     core::simd::LaneCount<L>: core::simd::SupportedLaneCount,
/// {
///     pub left: [core::simd::Simd<u32, L>; 2],
///     pub right: [core::simd::Simd<u32, L>; 2],
/// }
///
/// impl<const L: usize> ContainingStruct<L>
/// where
///     core::simd::LaneCount<L>: core::simd::SupportedLaneCount,
/// {
///     const fn base_offsets_in_lanes() -> [usize; 4] {
///         const SCALAR_SIZE_IN_BYTES: usize = core::mem::size_of::<u32>();
///
///         [
///             core::mem::offset_of!(Self, left) / SCALAR_SIZE_IN_BYTES,
///             (core::mem::offset_of!(Self, left) + Self::SIMD_SIZE)
///                 / SCALAR_SIZE_IN_BYTES,
///             core::mem::offset_of!(Self, right) / SCALAR_SIZE_IN_BYTES,
///             (core::mem::offset_of!(Self, right) + Self::SIMD_SIZE)
///                 / SCALAR_SIZE_IN_BYTES,
///         ]
///     }
///
///     const fn from_array(arr: [core::simd::Simd<u32, L>; 4]) -> Self {
///        Self {
///            left: [arr[0], arr[1]],
///            right: [arr[2], arr[3]],
///        }
///     }
///
///     // We can now invoke the macro as expected
///     aosoa::impl_aosoa!(u32, L, 4);
/// }
///
/// fn main() {
///     // We now have a `gather` method available on `ContainingStruct`, with the following signature:
///     // `pub fn gather(arr: &[Self], indices: core::simd::Simd<u32, L>) -> Self`
///     let buffer = &[
///         ContainingStruct{
///             left: [core::simd::Simd::<u32, 4>::splat(0), core::simd::Simd::<u32, 4>::splat(1)],
///             right: [core::simd::Simd::<u32, 4>::splat(2), core::simd::Simd::<u32, 4>::splat(3)]
///         },
///         ContainingStruct{
///             left: [core::simd::Simd::<u32, 4>::splat(4), core::simd::Simd::<u32, 4>::splat(5)],
///             right: [core::simd::Simd::<u32, 4>::splat(6), core::simd::Simd::<u32, 4>::splat(7)]
///         }
///     ];
///
///     let gathered = ContainingStruct::gather(buffer, core::simd::Simd::from([0, 1, 4, 5]));
///     // TODO: the assert
/// }
/// ```
#[macro_export]
macro_rules! impl_aosoa(
    ($scalar_type:ty, $lane_generic:ident, $field_count:expr) => {
        /// A shorthand for `core::mem::size_of::<Self>()`
        const STRUCT_SIZE: usize = core::mem::size_of::<Self>();

        /// The size in bytes of a simd element of the struct
        const SIMD_SIZE: usize = core::mem::size_of::<core::simd::Simd<$scalar_type, $lane_generic>>();

        /// The amount of these packed vectors that the struct contains
        const SIMD_COUNT: usize = $field_count;

        #[inline]
        fn compute_base_offset(indices: $crate::SimdU32<$lane_generic>) -> $crate::SimdU32<$lane_generic> {
            let lanes_splat = $crate::SimdU32::splat(u32::try_from($lane_generic).unwrap());
            let struct_index = indices / lanes_splat;
            let inner_index = indices % lanes_splat;

            let base = struct_index * $crate::SimdU32::splat(u32::try_from(Self::SIMD_COUNT * $lane_generic).unwrap()) + inner_index;
            base
        }

        #[inline]
        fn gather_simd_field_single(flattened: &[$scalar_type], base: $crate::SimdU32<$lane_generic>, field_order: usize) -> core::simd::Simd<$scalar_type, $lane_generic> {
            let offsets = Self::base_offsets_in_lanes();
            $crate::aosoa::gather_simd_field_single(flattened, base, offsets[field_order])
        }

        #[inline]
        fn gather_simd_field_all(flattened: &[$scalar_type], base: $crate::SimdU32<$lane_generic>) -> [core::simd::Simd<$scalar_type, $lane_generic>; $field_count] {
            let get_from_offset = |order| Self::gather_simd_field_single(flattened, base, order);
            let mut field_orders = [0; $field_count];
            for i in 0..$field_count {
                field_orders[i] = i;
            }

            field_orders.map(|order| get_from_offset(order))
        }

        /// Gathers into a contiguous slice of the struct by the given indices, returning a struct
        /// that contains the pointed-to elements.
        #[inline]
        pub fn gather(arr: &[Self], indices: $crate::SimdU32<$lane_generic>) -> Self {
            // This should be optimized out by the compiler.
            // I couldn't find a way to do a const assertion...
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
pub use impl_aosoa;

/// For a simple AoSoA layout where every SIMD type is directly on the struct
#[macro_export]
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

        $crate::aosoa::impl_aosoa!($scalar_type, $lane_generic, $field_count);
    }
);
pub use impl_aosoa_flat;
