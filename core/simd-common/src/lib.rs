#![no_std]
#![feature(portable_simd)]
#![allow(unexpected_cfgs)]
use core::simd;

pub mod aosoa;
pub mod gather;
pub mod select;

#[cfg(feature = "nalgebra")]
pub mod convert_nalgebra;

pub type SimdU32<const N: usize> = simd::Simd<u32, { N }>;
pub type SimdF32<const N: usize> = simd::Simd<f32, { N }>;
pub type SimdVec3F<const N: usize> = [SimdF32<N>; 3];
pub type SimdVec3U<const N: usize> = [SimdU32<N>; 3];
pub type SimdMask<const N: usize> = simd::Mask<i32, { N }>;

cfg_if::cfg_if! {
    if #[cfg(
        all(
            target_arch = "hexagon",
            target_feature="hvx-length128b"
        )
    )] {
        pub const MIN_NATIVE_VECTOR_LENGTH: usize = 1024;
    } else if #[cfg(
        any(
            all(
                any(target_arch = "x86", target_arch = "x86_64"),
                target_feature="avx512f"
            ),
            all(
                target_arch = "hexagon",
                target_feature="hvx-length64b"
            )
        )
    )] {
        pub const MIN_NATIVE_VECTOR_LENGTH: usize = 512;
    } else if #[cfg(
        any(
            all(
                any(target_arch = "x86", target_arch = "x86_64"),
                target_feature="avx"
            ),
            all(
                target_arch = "loongarch64",
                target_feature="lasx"
            )
        )
    )] {
        pub const MIN_NATIVE_VECTOR_LENGTH: usize = 256;
    } else if #[cfg(
        any(
            all(
                any(target_arch = "x86", target_arch = "x86_64"),
                target_feature="sse"
            ),
            all(
                any(target_arch = "arm", target_arch = "aarch64", target_arch = "arm64ec"),
                target_feature="neon"
            ),
            all(
                any(target_arch = "wasm32", target_arch = "wasm64"),
                target_feature="simd128"
            ),
            all(
                target_arch = "loongarch64",
                target_feature="lsx"
            ),
            all(
                any(target_arch = "mips",target_arch = "mips32r6", target_arch = "mips64", target_arch = "mips64r6"),
                target_feature="msa"
            ),
            all(
                any(target_arch = "riscv32", target_arch = "riscv64"),
                // See RISC-V v-spec 18.3 https://github.com/riscvarchive/riscv-v-spec/blob/master/v-spec.adoc#183-v-vector-extension-for-application-processors
                target_feature="v"
            ),
            all(
                any(target_arch = "powerpc", target_arch = "powerpc64"),
                target_feature="altivec"
            ),
            all(
                target_arch = "s390x",
                target_feature = "vector"
            ),
            all(
                target_arch = "csky",
                target_feature = "vdspv1"
            )
        )
    )] {
        pub const MIN_NATIVE_VECTOR_LENGTH: usize = 128;
    } else if #[cfg(
        all(
            target_arch = "sparc",
            target_feature = "vis"
        )
    )]{
        pub const MIN_NATIVE_VECTOR_LENGTH: usize = 64;
    } else {
        pub const MIN_NATIVE_VECTOR_LENGTH: usize = core::mem::size_of::<usize>() * 8;
    }
}

pub const fn min_lane_count_for_scalar_width(width: usize) -> usize {
    MIN_NATIVE_VECTOR_LENGTH.div_ceil(width)
}

pub const fn min_lane_count_for_type<T>() -> usize {
    min_lane_count_for_scalar_width(core::mem::size_of::<T>())
}

/// Minimum lane count supported by the platform, when using 32 bit types (f32, u32, i32)
pub const MIN_LANE_COUNT_32: usize = min_lane_count_for_scalar_width(32);

/// Minimum lane count supported by the platform, when using 64 bit types (f64, u64, i64)
pub const MIN_LANE_COUNT_64: usize = min_lane_count_for_scalar_width(64);

pub fn platform_vector_width() -> usize {
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
            MIN_NATIVE_VECTOR_LENGTH
        }
    }
}
