use core::simd::{LaneCount, SupportedLaneCount};

pub fn check_nans_simd_msg<const L: usize>(
    data: core::simd::Simd<f32, L>,
    context: &str,
    variable: &str,
) where
    LaneCount<L>: SupportedLaneCount,
{
    #[cfg(debug_assertions)]
    for i in 0..L {
        debug_assert!(
            (!data[i].is_nan()) && (!data[i].is_infinite()),
            "[NAN-FAILURE({context}: `{variable}`)] Got an unexpected NaN in lane {i}. Whole: {data:?}."
        );
    }
}

pub fn check_nans_simd_vec_msg_masked<const L: usize, const N: usize>(
    data: [core::simd::Simd<f32, L>; N],
    mask: core::simd::Mask<i32, L>,
    context: &str,
    variable: &str,
) where
    LaneCount<L>: SupportedLaneCount,
{
    #[cfg(debug_assertions)]
    for n in 0..N {
        for l in 0..L {
            if mask.test(l) {
                debug_assert!(!data[n][l].is_nan()  && !data[n][l].is_infinite(), "[NAN-FAILURE({context}: `{variable}`)] Got an unexpected NaN in member `{variable}[{n}]` lane {l}.");
            }
        }
    }
}

pub fn check_nans_simd_vec_msg<const L: usize, const N: usize>(
    data: [core::simd::Simd<f32, L>; N],
    context: &str,
    variable: &str,
) where
    LaneCount<L>: SupportedLaneCount,
{
    #[cfg(debug_assertions)]
    check_nans_simd_vec_msg_masked(data, core::simd::Mask::splat(true), context, variable);
}

pub(crate) fn check_passthrough<T>(value: T) -> bool {
    return true;
}

pub(crate) fn check_nonan(value: f32) -> bool {
    !value.is_nan()
}

pub(crate) fn check_noinf(value: f32) -> bool {
    !value.is_infinite()
}

pub(crate) fn check_nozero(value: f32) -> bool {
    value != 0.0
}

pub(crate) fn check_a<const L: usize, const N: usize, F>(
    data: [core::simd::Simd<f32, L>; N],
    mask: core::simd::Mask<i32, L>,
    check: F,
) -> Option<(usize, usize)>
where
    LaneCount<L>: SupportedLaneCount,
    F: Fn(f32) -> bool,
{
    for n in 0..N {
        for l in 0..L {
            if mask.test(l) && !check(data[n][l]) {
                return Some((n, l));
            }
        }
    }
    None
}

pub(crate) fn check_aw<const L: usize, const N: usize, F>(
    data: [simba::simd::Simd<core::simd::Simd<f32, L>>; N],
    mask: core::simd::Mask<i32, L>,
    check: F,
) -> Option<(usize, usize)>
where
    LaneCount<L>: SupportedLaneCount,
    F: Fn(f32) -> bool,
{
    for n in 0..N {
        for l in 0..L {
            if mask.test(l) && !check(data[n].0[l]) {
                return Some((n, l));
            }
        }
    }
    None
}

pub(crate) fn check_v3<const L: usize, F>(
    data: nalgebra::Vector3<simba::simd::Simd<core::simd::Simd<f32, L>>>,
    mask: core::simd::Mask<i32, L>,
    check: F,
) -> Option<(usize, usize)>
where
    LaneCount<L>: SupportedLaneCount,
    F: Fn(f32) -> bool,
{
    for n in 0..3 {
        for l in 0..L {
            if mask.test(l) && !check(data[n].0[l]) {
                return Some((n, l));
            }
        }
    }
    None
}

pub(crate) fn check_s<const L: usize, F>(
    data: core::simd::Simd<f32, L>,
    mask: core::simd::Mask<i32, L>,
    check: F,
) -> Option<usize>
where
    LaneCount<L>: SupportedLaneCount,
    F: Fn(f32) -> bool,
{
    for l in 0..L {
        if mask.test(l) && !check(data[l]) {
            return Some(l);
        }
    }

    None
}

pub(crate) fn check_sw<const L: usize, F>(
    data: simba::simd::Simd<core::simd::Simd<f32, L>>,
    mask: core::simd::Mask<i32, L>,
    check: F,
) -> Option<usize>
where
    LaneCount<L>: SupportedLaneCount,
    F: Fn(f32) -> bool,
{
    check_s(data.0, mask, check)
}
// Goal, create a macro that:
// - Checks that an expression upholds the given things
//   - It might be a single SIMD (scalar) or a vector of SIMDs
// - If it doesn't, then it reports the guilty lane
// - You can also give it the dependencies of the expression
macro_rules! ensure_simd(
    (@checker_for_property passthrough) => ($crate::kernels::operations::debug::check_passthrough);
    (@checker_for_property nonan) => ($crate::kernels::operations::debug::check_nonan);
    (@checker_for_property noinf) => ($crate::kernels::operations::debug::check_noinf);
    (@checker_for_property nozero) => ($crate::kernels::operations::debug::check_nozero);

    (@checker_for_properties $($property:tt),+) => {
        |v| {
            $(
                $crate::kernels::operations::debug::ensure_simd!(@checker_for_property $property)(v)
            )&&+
        }
    };
    (@check v3, @upholds($($property:tt),+), $data:expr, $mask:expr) => ($crate::kernels::operations::debug::check_v3($data, $mask, $crate::kernels::operations::debug::ensure_simd!(@checker_for_properties $($property),+)).map(|s| s.1));
    (@check aw, @upholds($($property:tt),+), $data:expr, $mask:expr) => ($crate::kernels::operations::debug::check_aw($data, $mask, $crate::kernels::operations::debug::ensure_simd!(@checker_for_properties $($property),+)).map(|s| s.1));
    (@check a, @upholds($($property:tt),+), $data:expr, $mask:expr) => ($crate::kernels::operations::debug::check_a($data, $mask, $crate::kernels::operations::debug::ensure_simd!(@checker_for_properties $($property),+)).map(|s| s.1));
    (@check s, @upholds($($property:tt),+), $data:expr, $mask:expr) => ($crate::kernels::operations::debug::check_s($data, $mask, $crate::kernels::operations::debug::ensure_simd!(@checker_for_properties $($property),+)));
    (@check sw, @upholds($($property:tt),+), $data:expr, $mask:expr) => ($crate::kernels::operations::debug::check_sw($data, $mask, $crate::kernels::operations::debug::ensure_simd!(@checker_for_properties $($property),+)));

    ($expr:expr; $shape:tt; @mask($mask:expr); @upholds($($property:tt),+); @depends($($dependency:expr),*)) => {
        {
            let expr = $expr;

            #[cfg(debug_assertions)]
            if let Some(violating_lane) = $crate::kernels::operations::debug::ensure_simd!(@check $shape, @upholds($($property),+), expr, $mask) {
                panic!(concat!(
                    "`",
                    stringify!($expr),
                    "`: properties [",
                    $(
                        "\"",
                        stringify!($property),
                        "\", ",
                    )+
                    "] not upholded (lane {}): {:?}.",
                    " Dependencies: ",
                    $(
                        "\n`",
                        stringify!($dependency),
                        "`: {:?}, ",
                    )*
                ), violating_lane, expr, $($dependency),*);
            }

            expr
        }
    };

    ($expr:expr; $shape:tt; @upholds($($property:tt),+)) => {
        {
            #[cfg(debug_assertions)]
            let mask = core::simd::Mask::splat(true);
            $crate::kernels::operations::debug::ensure_simd!($expr; $shape; @mask(mask); @upholds($($property),+); @depends())
        }
    };


    ($expr:expr; $shape:tt; @mask($mask:expr)) => {
        {
            $crate::kernels::operations::debug::ensure_simd!($expr; $shape; @mask($mask); @upholds(nonan, noinf); @depends())
        }
    };

    ($expr:expr; $shape:tt; @depends($($dependency:expr),*)) => {
        {
            #[cfg(debug_assertions)]
            let mask = core::simd::Mask::splat(true);
            $crate::kernels::operations::debug::ensure_simd!($expr; $shape; @mask(mask); @upholds(nonan, noinf); @depends($($dependency),*))
        }
    };

    ($expr:expr; $shape:tt) => {
        {
            $crate::kernels::operations::debug::ensure_simd!($expr; $shape; @upholds(nonan, noinf))
        }
    };
);
pub(crate) use ensure_simd;
