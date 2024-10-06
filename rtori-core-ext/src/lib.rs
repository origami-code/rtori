#![feature(allocator_api)]
#![feature(ptr_as_uninit)]
#![feature(maybe_uninit_write_slice)]
#![deny(unsafe_op_in_unsafe_fn)]

mod allocator;
pub use allocator::*;

mod context;
pub use context::*;

mod solver;
pub use solver::*;

mod fold;
pub use fold::*;

mod output;
pub use output::*;

use rtori_core::model;

type Arc<'alloc, T> = std::sync::Arc<T, ContextAllocator<'alloc>>;
