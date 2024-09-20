#![feature(allocator_api)]

use std::alloc::Allocator;
use std::sync::Arc;

pub mod os_solver;

pub struct Context<A> {
    allocator: A
}

impl<A: Allocator> Context<A> {
    pub const fn new(allocator: A) -> Self {
        Self {
            allocator
        }
    }
}

impl<A> Context<A> {
    pub async fn create_os_solver(
        &self,
        backends: os_solver::BackendFlags
    ) -> Result<os_solver::Solver, ()> {
        os_solver::Solver::create(backends).await
    }
}
