#![no_std]
#![feature(allocator_api)]

use core::alloc::Allocator;
extern crate alloc;
pub mod os_solver;

pub use fold;

pub use rtori_os_fold_importer as fold_importer;
pub use rtori_os_model as model;

use bitflags::bitflags;
bitflags! {
    #[derive(Copy, Clone, Debug, PartialEq, PartialOrd)]
    #[repr(C)]
    pub struct BackendFlags: u8 {
        const CPU = 1 << 0;
        const CPU_MT = 1 << 1;

        const GPU_METAL = 1 << 3;
        const GPU_VULKAN = 1 << 4;
        const GPU_DX12 = 1 << 5;
        const GPU_WEBGPU = 1 << 6;

        const GPU_ANY = BackendFlags::GPU_METAL.bits() | BackendFlags::GPU_VULKAN.bits() | BackendFlags::GPU_DX12.bits() | BackendFlags::GPU_WEBGPU.bits();
        const ANY = BackendFlags::GPU_ANY.bits() | BackendFlags::CPU.bits() | BackendFlags::CPU_MT.bits();
    }
}

impl Default for BackendFlags {
    fn default() -> Self {
        BackendFlags::ANY
    }
}

#[derive(Debug)]
pub enum SolverFamily {
    OrigamiSimulator,
}

#[derive(Debug)]
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

#[derive(Debug)]
pub struct Solver<'ctx, A>
where
    A: Allocator,
{
    context: &'ctx Context<A>,
    pub inner: SolverKind,
}

impl<'ctx, A> Solver<'ctx, A>
where
    A: Allocator,
{
    pub async fn create(
        ctx: &'ctx Context<A>,
        family: SolverFamily,
        backend: BackendFlags,
    ) -> Result<Solver<'ctx, A>, ()> {
        match family {
            SolverFamily::OrigamiSimulator => {
                os_solver::Solver::create(backend).await.map(|inner| Self {
                    context: ctx,
                    inner: SolverKind::OS(inner),
                })
            }
        }
    }

    pub fn load_fold_in(&mut self, frame: &fold::FrameCore) -> Result<(), ()> {
        match &mut self.inner {
            SolverKind::OS(solver) => solver.load_fold_in(frame, &self.context.allocator),
        }

        Ok(())
    }

    pub fn loaded(&self) -> bool {
        match &self.inner {
            SolverKind::OS(s) => s.loaded(),
        }
    }
}

#[derive(Debug)]
pub struct Context<A> {
    allocator: A,
}

impl<A: Allocator> Context<A> {
    pub const fn new(allocator: A) -> Self {
        Self { allocator }
    }
}
