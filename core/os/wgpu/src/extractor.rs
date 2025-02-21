use std::{marker::PhantomData, num::NonZeroU64, ops::Deref};

use crate::loader::LoaderRange;

#[derive(Debug)]
pub(crate) struct ExtractorRanges {
    pub node_position_offset: Option<LoaderRange>,
    pub node_error: Option<LoaderRange>,
}

#[derive(Debug)]
pub struct ExtractorMappedTarget<'a> {
    buffer: &'a wgpu::Buffer,
    map: ExtractorRanges,
}

macro_rules! define_map_field {
    ($func:ident, $field:ident) => {
        pub fn $func(&'a self) -> Option<wgpu::BufferView<'a>> {
            self.map.$field.map(|range| self.map_inner(range))
        }
    };
}

impl<'a> ExtractorMappedTarget<'a> {
    pub(crate) fn new(buffer: &'a wgpu::Buffer, map: ExtractorRanges) -> Self {
        Self { buffer, map }
    }

    fn map_inner(&'a self, range: LoaderRange) -> wgpu::BufferView<'a> {
        let range = range.offset..(range.offset + range.size.get());
        let slice: wgpu::BufferSlice<'_> = self.buffer.slice(range);
        let view = slice.get_mapped_range();
        view
    }

    define_map_field! {map_node_position_offset, node_position_offset}
    define_map_field! {map_node_error, node_error}
}

type Proxy<'a, T> = rtori_os_model::proxy::Proxy<wgpu::BufferView<'a>, T>;

macro_rules! define_mappable_pair {
    ($associated_type:ident, $inner_type:ty, $fn_name:ident) => {
        type $associated_type<'a>
            = Proxy<'a, $inner_type>
        where
            Self: 'a,
            'a: 'container;
        fn $fn_name<'call>(&'call self) -> Option<Self::$associated_type<'call>>
        where
            'call: 'container,
        {
            self.$fn_name().map(|inner| Proxy::new(inner))
        }
    };
}

impl<'container> rtori_os_model::MappedResults<'container> for ExtractorMappedTarget<'container> {
    define_mappable_pair!(
        NodePositionOffsetMap,
        rtori_os_model::Vector3F,
        map_node_position_offset
    );
    define_mappable_pair!(NodeErrorMap, f32, map_node_error);
}
