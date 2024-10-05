#![feature(allocator_api)]
#![deny(unsafe_op_in_unsafe_fn)]

mod allocator;
pub use allocator::*;

mod context;
pub use context::*;

mod solver;
pub use solver::*;

use rtori_core::model;

type Arc<'alloc, T> = std::sync::Arc<T, ContextAllocator<'alloc>>;
