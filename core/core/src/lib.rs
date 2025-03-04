#![no_std]
#![feature(allocator_api)]

use core::alloc::Allocator;
extern crate alloc;
pub mod os_solver;

pub use fold;

pub use rtori_os_fold_importer as fold_importer;
pub use rtori_os_model as model;

pub enum SolverKind {
    OS(os_solver::Solver),
}

pub enum SolverPreprocessedData<'a, A>
where
    A: Allocator,
{
    /// Preprocessing data for the Origami Simulator family of solvers
    OS(fold_importer::supplement::SupplementedInput<'a, A>),
}

pub struct Solver<'ctx, A>
where
    A: Allocator,
{
    context: &'ctx Context<A>,
    inner: SolverKind,
}

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
