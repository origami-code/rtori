use core::simd;

pub type SimdU32N<const N: usize> = simd::Simd<u32, { N }>;
pub type SimdF32N<const N: usize> = simd::Simd<f32, { N }>;
pub type SimdVec3FN<const N: usize> = [SimdF32N<N>; 3];

#[allow(dead_code)]
mod config_16 {
    pub const CHUNK_SIZE: usize = 16;
}

#[allow(dead_code)]
mod config_8 {
    pub const CHUNK_SIZE: usize = 8;
}

#[allow(dead_code)]
mod config_4 {
    pub const CHUNK_SIZE: usize = 4;
}

#[allow(dead_code)]
mod config_2 {
    pub const CHUNK_SIZE: usize = 2;
}

#[allow(dead_code)]
mod config_1 {
    pub const CHUNK_SIZE: usize = 1;
}

cfg_if::cfg_if! {
    if #[cfg(
        all(
            any(target_arch = "x86", target_arch = "x86_64"),
            target_feature="avx512f"
        )
    )] {
        pub use config_16::*;
    } else if #[cfg(
        all(
            any(target_arch = "x86", target_arch = "x86_64"),
            target_feature="avx"
        )
    )] {
        pub use config_8::*;
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
        pub use config_4::*;
    } else {
        pub use config_1::*;
    }
}

pub type SimdF32 = SimdF32N<{ CHUNK_SIZE }>;
pub type SimdU32 = SimdU32N<{ CHUNK_SIZE }>;
pub type SimdMask = simd::Mask<i32, { CHUNK_SIZE }>;
pub type SimdVec3F = [SimdF32; 3];
pub type SimdVec3U = [SimdU32; 3];
