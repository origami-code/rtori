#[cfg(feature = "alloc")]
extern crate alloc;

use core::{
    alloc::Allocator,
    marker::PhantomData,
    simd::{LaneCount, SupportedLaneCount},
};

// This is a self-referential struct
// Hence, we need to
#[derive(Debug)]
pub struct OwnedRunner<'allocator, const L: usize, A>
where
    LaneCount<L>: SupportedLaneCount,
    A: Allocator + 'allocator,
{
    runner: crate::Runner<'allocator, L>,
    buffer: alloc::boxed::Box<[u8], A>,
    _allocator_marker: PhantomData<&'allocator A>,
}

impl<'allocator, const L: usize, A> OwnedRunner<'allocator, L, A>
where
    LaneCount<L>: SupportedLaneCount,
    A: Allocator + 'allocator,
{
    pub const fn runner(&self) -> &crate::Runner<'allocator, L> {
        &self.runner
    }

    pub const fn runner_mut(&mut self) -> &mut crate::Runner<'allocator, L> {
        &mut self.runner
    }
}

impl<'allocator, const L: usize, A> OwnedRunner<'allocator, L, A>
where
    LaneCount<L>: SupportedLaneCount,
    simba::simd::Simd<core::simd::Simd<f32, L>>: simba::simd::SimdRealField,
    A: Allocator + 'allocator,
{
    /// Consumes the runner, returning the owned buffer in the process (for reuse)
    pub fn consume(self) -> alloc::boxed::Box<[u8], A> {
        self.buffer
    }

    pub fn with_buffer(
        size: &rtori_os_model::ModelSize,
        mut buffer: alloc::boxed::Box<[u8], A>,
    ) -> Result<Self, ()> {
        let buffer_size = crate::Runner::query_backing_size_requirement(size);
        if buffer.len() < buffer_size {
            return Err(());
        }

        let transmuted =
            unsafe { core::mem::transmute::<&mut [u8], &'allocator mut [u8]>(buffer.as_mut()) };
        let (runner, _rest): (crate::Runner<'allocator, L>, _) =
            crate::Runner::from_backing_slice(size, transmuted)
                .expect("Should not fail as we called `query_backing_size_requirement` beforehand");

        Ok(Self {
            runner,
            buffer,
            _allocator_marker: PhantomData,
        })
    }

    pub fn with_size_in(size: &rtori_os_model::ModelSize, allocator: A) -> Self {
        let buffer_size = crate::Runner::query_backing_size_requirement(size);
        let buffer = {
            let mut buf = alloc::vec::Vec::new_in(allocator);
            buf.resize(buffer_size, 0u8);
            buf.into_boxed_slice()
        };

        Self::with_buffer(size, buffer).expect("should never fail as we check the buffer size here")
    }
}

impl<const L: usize> OwnedRunner<'static, L, alloc::alloc::Global>
where
    LaneCount<L>: SupportedLaneCount,
    simba::simd::Simd<core::simd::Simd<f32, L>>: simba::simd::SimdRealField,
{
    pub fn with_size(size: &rtori_os_model::ModelSize) -> Self {
        Self::with_size_in(size, alloc::alloc::Global)
    }
}

impl<'allocator, const L: usize, A> core::ops::Deref for OwnedRunner<'allocator, L, A>
where
    LaneCount<L>: SupportedLaneCount,
    A: Allocator + 'allocator,
{
    type Target = crate::Runner<'allocator, L>;

    fn deref(&self) -> &Self::Target {
        self.runner()
    }
}

impl<'allocator, const L: usize, A> core::ops::DerefMut for OwnedRunner<'allocator, L, A>
where
    LaneCount<L>: SupportedLaneCount,
    A: Allocator + 'allocator,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.runner_mut()
    }
}
