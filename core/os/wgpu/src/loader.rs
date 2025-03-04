use std::{marker::PhantomData, num::NonZeroU64, ops::Deref};

#[derive(Debug, Clone, Copy)]
pub(crate) struct LoaderRange {
    pub offset: u64,
    pub size: NonZeroU64,
}

#[derive(Debug)]
pub(crate) struct LoadRanges {
    pub node_positions_unchanging: LoaderRange,
    pub node_external_forces: LoaderRange,
    pub node_configs: LoaderRange,
    pub node_geometry: LoaderRange,
    pub crease_geometry: LoaderRange,
    pub crease_parameters: LoaderRange,
    pub face_indices: LoaderRange,
    pub face_nominal_angles: LoaderRange,
    pub node_creases: LoaderRange,
    pub node_beams: LoaderRange,
    pub node_faces: LoaderRange,
}

#[derive(Debug)]
pub struct LoaderMappedTarget<'a> {
    buffer: &'a wgpu::Buffer,
    map: LoadRanges,
}

macro_rules! define_map_field {
    (@ref, $func:ident, $field:ident) => {
        pub fn $func(&'a self) -> wgpu::BufferViewMut<'a> {
            self.map_inner(self.map.$field)
        }
    };

    (@mut, $func:ident, $field:ident) => {
        pub fn $func(&'a mut self) -> wgpu::BufferViewMut<'a> {
            self.map_inner(self.map.$field)
        }
    };
}

impl<'a> LoaderMappedTarget<'a> {
    pub(crate) fn new(buffer: &'a wgpu::Buffer, map: LoadRanges) -> Self {
        Self { buffer, map }
    }

    fn map_inner(&'a self, range: LoaderRange) -> wgpu::BufferViewMut<'a> {
        let range = range.offset..(range.offset + range.size.get());
        let slice: wgpu::BufferSlice<'_> = self.buffer.slice(range);
        let view_mut = slice.get_mapped_range_mut();
        view_mut
    }

    define_map_field! {@ref, map_node_position, node_positions_unchanging}
    define_map_field! {@ref, map_node_external_forces, node_external_forces}
    define_map_field! {@ref, map_node_config, node_configs}
    define_map_field! {@ref, map_node_geometry, node_geometry}
    define_map_field! {@ref, map_crease_geometry, crease_geometry}
    define_map_field! {@ref, map_crease_parameters, crease_parameters}
    define_map_field! {@ref, map_face_indices, face_indices}
    define_map_field! {@ref, map_face_nominal_angles, face_nominal_angles}
    define_map_field! {@ref, map_node_crease, node_creases}
    define_map_field! {@ref, map_node_beam, node_beams}
    define_map_field! {@ref, map_node_face, node_faces}
}

#[derive(Debug)]
pub struct LoaderStagingBelt<'a> {
    target: &'a wgpu::Buffer,
    encoder: &'a mut wgpu::CommandEncoder,
    belt: &'a mut wgpu::util::StagingBelt,
    device: &'a wgpu::Device,
    map: LoadRanges,
}

impl<'a> LoaderStagingBelt<'a> {
    pub(crate) fn new(
        target: &'a wgpu::Buffer,
        encoder: &'a mut wgpu::CommandEncoder,
        belt: &'a mut wgpu::util::StagingBelt,
        device: &'a wgpu::Device,
        map: LoadRanges,
    ) -> Self {
        Self {
            target,
            encoder,
            belt,
            device,
            map,
        }
    }

    fn map_inner(&'a mut self, range: LoaderRange) -> wgpu::BufferViewMut<'a> {
        let view_mut: wgpu::BufferViewMut<'a> = self.belt.write_buffer(
            self.encoder,
            self.target,
            range.offset,
            range.size,
            self.device,
        );
        view_mut
    }

    define_map_field! {@mut, map_node_position, node_positions_unchanging}
    define_map_field! {@mut, map_node_external_forces, node_external_forces}
    define_map_field! {@mut, map_node_config, node_configs}
    define_map_field! {@mut, map_node_geometry, node_geometry}
    define_map_field! {@mut, map_crease_geometry, crease_geometry}
    define_map_field! {@mut, map_crease_parameters, crease_parameters}
    define_map_field! {@mut, map_face_indices, face_indices}
    define_map_field! {@mut, map_face_nominal_angles, face_nominal_angles}
    define_map_field! {@mut, map_node_crease, node_creases}
    define_map_field! {@mut, map_node_beam, node_beams}
    define_map_field! {@mut, map_node_face, node_faces}
}

macro_rules! dispatch_map_field {
    ($func:ident) => {
        pub fn $func(&'a mut self) -> wgpu::BufferViewMut<'a> {
            match self {
                Self::Mapped(inner) => inner.$func(),
                Self::StagingBelt(inner) => inner.$func(),
            }
        }
    };
}

#[derive(Debug)]
pub enum Loader<'a> {
    Mapped(LoaderMappedTarget<'a>),
    StagingBelt(LoaderStagingBelt<'a>),
}

impl<'a> Loader<'a> {
    dispatch_map_field! {map_node_position}
    dispatch_map_field! {map_node_external_forces}
    dispatch_map_field! {map_node_config}
    dispatch_map_field! {map_node_geometry}
    dispatch_map_field! {map_crease_geometry}
    dispatch_map_field! {map_crease_parameters}
    dispatch_map_field! {map_face_indices}
    dispatch_map_field! {map_face_nominal_angles}
    dispatch_map_field! {map_node_crease}
    dispatch_map_field! {map_node_beam}
    dispatch_map_field! {map_node_face}
}

type Proxy<'a, T> = rtori_os_model::proxy::Proxy<wgpu::BufferViewMut<'a>, T>;

macro_rules! define_mappable_pair {
    ($associated_type:ident, $inner_type:ty, $fn_name:ident) => {
        type $associated_type<'a>
            = Proxy<'a, $inner_type>
        where
            Self: 'a,
            'a: 'container;
        fn $fn_name<'call>(&'call mut self) -> Self::$associated_type<'call>
        where
            'call: 'container,
        {
            Proxy::new(self.$fn_name())
        }
    };
}

impl<'container> rtori_os_model::MappedDestination<'container> for Loader<'container> {
    define_mappable_pair!(NodePositionMap, rtori_os_model::Vector3F, map_node_position);
    define_mappable_pair!(
        NodeExternalForcesMap,
        rtori_os_model::Vector3F,
        map_node_external_forces
    );
    define_mappable_pair!(NodeConfigMap, rtori_os_model::NodeConfig, map_node_config);
    define_mappable_pair!(
        NodeGeometryMap,
        rtori_os_model::NodeGeometry,
        map_node_geometry
    );
    define_mappable_pair!(
        CreaseGeometryMap,
        rtori_os_model::CreaseGeometry,
        map_crease_geometry
    );
    define_mappable_pair!(
        CreaseParametersMap,
        rtori_os_model::CreaseParameters,
        map_crease_parameters
    );
    define_mappable_pair!(FaceIndicesMap, rtori_os_model::Vector3U, map_face_indices);
    define_mappable_pair!(
        FaceNominalAnglesMap,
        rtori_os_model::Vector3F,
        map_face_nominal_angles
    );
    define_mappable_pair!(
        NodeCreaseMap,
        rtori_os_model::NodeCreaseSpec,
        map_node_crease
    );
    define_mappable_pair!(NodeBeamMap, rtori_os_model::NodeBeamSpec, map_node_beam);
    define_mappable_pair!(NodeFaceMap, rtori_os_model::NodeFaceSpec, map_node_face);
}
