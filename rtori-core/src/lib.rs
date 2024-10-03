#![no_std]
#![feature(allocator_api)]

use core::alloc::Allocator;
extern crate alloc;
pub mod os_solver;

pub use fold;

pub struct Context<A> {
    allocator: A,
}

impl<A: Allocator> Context<A> {
    pub const fn new(allocator: A) -> Self {
        Self { allocator }
    }
}

impl<A> Context<A> {
    pub async fn create_os_solver(
        &self,
        backends: os_solver::BackendFlags,
    ) -> Result<os_solver::Solver, ()> {
        os_solver::Solver::create(backends).await
    }
}
