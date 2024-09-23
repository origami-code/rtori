use std::{marker::PhantomData, num::NonZeroU64, ops::Deref};

use crate::loader::LoaderRange;

#[derive(Debug)]
pub(crate) struct BufferTarget<'a> {
    pub buffer: &'a wgpu::Buffer,
    pub offset: u64,
    pub length: NonZeroU64,
}

#[derive(Debug)]
pub(crate) struct ExtractorGPUTarget<'a> {
    pub node_position_offset: Option<BufferTarget<'a>>,
    pub node_error: Option<BufferTarget<'a>>,
    pub node_velocity: Option<BufferTarget<'a>>,
}
