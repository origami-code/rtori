use core::simd;

#[allow(dead_code)]
mod config_16 {
    use super::simd;
    pub const CHUNK_SIZE: usize = 16;
    pub type SimdF32 = simd::f32x16;
    pub type SimdU32 = simd::u32x16;
    pub type SimdMask = u16;
}

#[allow(dead_code)]
mod config_8 {
    use super::simd;
    pub const CHUNK_SIZE: usize = 8;
    pub type SimdF32 = simd::f32x8;
    pub type SimdU32 = simd::u32x8;
    pub type SimdMask = u8;
}

#[allow(dead_code)]
mod config_4 {
    use super::simd;
    pub const CHUNK_SIZE: usize = 4;
    pub type SimdF32 = simd::f32x4;
    pub type SimdU32 = simd::u32x4;
    pub type SimdMask = u8;
}

#[allow(dead_code)]
mod config_2 {
    use super::simd;
    pub const CHUNK_SIZE: usize = 2;
    pub type SimdF32 = simd::f32x2;
    pub type SimdU32 = simd::u32x2;
    pub type SimdMask = u8;
}

#[allow(dead_code)]
mod config_1 {
    use super::simd;
    pub const CHUNK_SIZE: usize = 1;
    pub type SimdF32 = simd::f32x1;
    pub type SimdU32 = simd::u32x1;
    pub type SimdMask = u8;
}


cfg_if::cfg_if!{
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

pub type SimdVec3F = [SimdF32; 3];
pub type SimdVec3U = [SimdU32; 3];
