use core::simd;

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
        pub const NATIVE_VECTOR_LENGTH: usize = 1024;
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
        pub const NATIVE_VECTOR_LENGTH: usize = 512;
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
        pub const NATIVE_VECTOR_LENGTH: usize = 256;
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
        pub const NATIVE_VECTOR_LENGTH: usize = 128;
    } else if #[cfg(
        all(
            target_arch = "sparc",
            target_feature = "vis"
        )    
    )]{
        pub const NATIVE_VECTOR_LENGTH: usize = 64;
    } else {
        pub const NATIVE_VECTOR_LENGTH: usize = core::mem::size_of::<usize>() * 8;
    }
}

pub const PREFERRED_WIDTH: usize = NATIVE_VECTOR_LENGTH / 32;
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
