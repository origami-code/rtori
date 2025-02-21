#![no_std] // temporary for testing
#![feature(portable_simd)]
#![feature(iter_array_chunks)]
#![feature(stmt_expr_attributes)]
#![feature(impl_trait_in_assoc_type)]
#![feature(type_alias_impl_trait)]
#![cfg_attr(feature = "alloc", feature(allocator_api))]

extern crate static_assertions;

mod kernels;
mod model;
mod process;

mod runner;
pub use runner::*;

#[cfg(feature = "alloc")]
pub mod owned;

mod extractor;
pub use extractor::Extractor;

mod loader;
pub use loader::Loader;
