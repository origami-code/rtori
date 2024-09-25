#![no_std]
#![feature(portable_simd)]
#![feature(iter_array_chunks)]
#![feature(stmt_expr_attributes)]
#![feature(impl_trait_in_assoc_type)]
mod kernels;
mod model;
mod simd_atoms;
