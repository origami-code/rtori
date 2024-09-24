#![no_std]
#![feature(portable_simd)]
#![feature(iter_array_chunks)]
#![feature(stmt_expr_attributes)]

mod kernels;
mod model;
mod simd_atoms;
