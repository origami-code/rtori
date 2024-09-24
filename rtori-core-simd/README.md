#

Goal: compile with several different feature sets

x86_64 & x86
see for the support <https://store.steampowered.com/hwsurvey>
see for the names <https://github.com/HenrikBengtsson/x86-64-level>

- (x86-64-v2) 99.6% SSE4.2 [+SSSE3,SSE4.1,SSE3, SSE2] (f32 -> 4-wide operations)
- (x86-64-v3) 95% AVX2 [+AVX, +FCMOV] (f32 -> 8-wide operations)
- (x86-64-v4) AVX512 [AVX512F] (f32 -> 16-wide operations)

armv7

- NEON vfpv=neon (f32 -> 4-wide operations)
- NEON vfpv=neon-vpfv4 (same)

armv8 and later

- NEON vfpv=neon-fp-armv8 (same as armv7)

aarch64 (by default, nothing to be done)

wasm32

- SIMD ? (f32 -> 4-wide)
