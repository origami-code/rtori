use core::simd;

pub type SimdU32<const N: usize> = simd::Simd<u32, { N }>;
pub type SimdF32<const N: usize> = simd::Simd<f32, { N }>;
pub type SimdVec3F<const N: usize> = [SimdF32<N>; 3];
pub type SimdVec3U<const N: usize> = [SimdU32<N>; 3];
pub type SimdMask<const N: usize> = simd::Mask<i32, { N }>;

cfg_if::cfg_if! {
    if #[cfg(
        all(
            any(target_arch = "x86", target_arch = "x86_64"),
            target_feature="avx512f"
        )
    )] {
        pub const PREFERRED_WIDTH: usize = 16;
    } else if #[cfg(
        all(
            any(target_arch = "x86", target_arch = "x86_64"),
            target_feature="avx"
        )
    )] {
        pub const PREFERRED_WIDTH: usize = 8;
    } else if #[cfg(
        any(
            all(
                any(target_arch = "x86", target_arch = "x86_64"),
                target_feature="sse"
            ),
            all(
                target_arch = "arm",
                target_feature="neon"
            ),
            target_arch = "aarch64", // implies neon as it's mandatory
            all(
                any(target_arch = "wasm32", target_arch = "wasm64"),
                target_feature="simd128"
            )
        )
    )] {
        pub const PREFERRED_WIDTH: usize = 4;
    } else {
        pub const PREFERRED_WIDTH: usize = 1;
    }
}

static_assertions::assert_impl_all!(simba::simd::Simd<SimdF32<{PREFERRED_WIDTH}>>: nalgebra::SimdRealField);

pub fn preferred_width() -> usize {
    cfg_if::cfg_if! {
        if #[cfg(
            all(target_arch = "aarch64", target_feature = "sve")
        )] {
            let res: u64 = unsafe {
                let mut res: u64 = 0;
                asm!(
                    "cntb {x}, ALL"
                    x = inout(reg) res
                );
                res
            };
            usize::try_from(res).unwrap()
        } else {
            PREFERRED_WIDTH
        }
    }
}
