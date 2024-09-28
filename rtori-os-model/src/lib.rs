#![no_std]
use core::ops::{Deref, DerefMut};

#[cfg(feature = "define_proxy")]
pub mod proxy;

pub type NodeIndex = u32;
pub type CreaseIndex = u32;
pub type FaceIndex = u32;
pub type NodeCreaseIndex = u32;
pub type NodeBeamIndex = u32;
pub type NodeFaceIndex = u32;

#[cfg_attr(feature = "bytemuck", derive(bytemuck::Pod, bytemuck::Zeroable))]
#[derive(Debug, Clone, Copy, PartialEq)]
#[repr(transparent)]
pub struct Vector3F(pub [f32; 3]);

#[cfg_attr(feature = "bytemuck", derive(bytemuck::Pod, bytemuck::Zeroable))]
#[derive(Debug, Clone, Copy, PartialEq)]
#[repr(transparent)]
pub struct Vector3U(pub [u32; 3]);

#[cfg_attr(feature = "bytemuck", derive(bytemuck::Pod, bytemuck::Zeroable))]
#[derive(Debug, Clone, Copy, PartialEq)]
#[repr(transparent)]
pub struct Vector2U(pub [u32; 3]);

#[cfg_attr(feature = "bytemuck", derive(bytemuck::Pod, bytemuck::Zeroable))]
#[derive(Debug, Clone, Copy, PartialEq)]
#[repr(C)]
pub struct NodeConfig {
    pub mass: f32,
    pub fixed: u8,
    pub _reserved: [u8; 3],
}

#[cfg_attr(feature = "bytemuck", derive(bytemuck::Pod, bytemuck::Zeroable))]
#[derive(Debug, Clone, Copy, PartialEq)]
#[repr(C)]
pub struct NodeCreasePointer {
    pub offset: NodeCreaseIndex,
    pub count: u32,
}

#[cfg_attr(feature = "bytemuck", derive(bytemuck::Pod, bytemuck::Zeroable))]
#[derive(Debug, Clone, Copy, PartialEq)]
#[repr(C)]
pub struct NodeBeamPointer {
    pub offset: NodeBeamIndex,
    pub count: u32,
}

#[cfg_attr(feature = "bytemuck", derive(bytemuck::Pod, bytemuck::Zeroable))]
#[derive(Debug, Clone, Copy, PartialEq)]
#[repr(C)]
pub struct NodeFacePointer {
    pub offset: NodeFaceIndex,
    pub count: u32,
}

#[cfg_attr(feature = "bytemuck", derive(bytemuck::Pod, bytemuck::Zeroable))]
#[derive(Debug, Clone, Copy, PartialEq)]
#[repr(C)]
pub struct NodeGeometry {
    pub crease: NodeCreasePointer,
    pub beam: NodeBeamPointer,
    pub face: NodeFacePointer,
}

#[cfg_attr(feature = "bytemuck", derive(bytemuck::Pod, bytemuck::Zeroable))]
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct CreaseGeometryFace {
    pub face_index: u32,
    pub complement_vertex_index: u32,
}

#[cfg_attr(feature = "bytemuck", derive(bytemuck::Pod, bytemuck::Zeroable))]
#[derive(Debug, Clone, Copy)]
#[repr(transparent)]
pub struct CreaseGeometry {
    pub faces: [CreaseGeometryFace; 2],
}

#[cfg_attr(feature = "bytemuck", derive(bytemuck::Pod, bytemuck::Zeroable))]
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct CreaseParameters {
    pub k: f32,
    pub d: f32,
    pub target_fold_angle: f32,
}

#[cfg_attr(feature = "bytemuck", derive(bytemuck::Pod, bytemuck::Zeroable))]
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct NodeCreaseSpec {
    pub crease_index: CreaseIndex,
    pub node_number: u32,
}

#[cfg_attr(feature = "bytemuck", derive(bytemuck::Pod, bytemuck::Zeroable))]
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct NodeBeamSpec {
    pub node_index: NodeIndex,
    pub k: f32,
    pub d: f32,
    pub length: f32,
    pub neighbour_index: u32,
}

#[cfg_attr(feature = "bytemuck", derive(bytemuck::Pod, bytemuck::Zeroable))]
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct NodeFaceSpec {
    pub node_index: NodeIndex,
    pub face_index: FaceIndex,
}

pub trait Destination {
    fn set_node_position(&mut self, node_idx: NodeIndex, pos: Vector3F);
    fn set_node_external_forces(&mut self, node_idx: NodeIndex, pos: Vector3F);
    fn set_node_config(&mut self, node_idx: NodeIndex, config: NodeConfig);
    fn set_node_geometry(&mut self, node_idx: NodeIndex, geometry: NodeGeometry);

    fn set_crease_geometry(&mut self, crease_idx: CreaseIndex, geometry: CreaseGeometry);
    fn set_crease_parameters(&mut self, crease_idx: CreaseIndex, parameters: CreaseParameters);

    fn set_face_indices(&mut self, face_idx: FaceIndex, node_indices: Vector3U);
    fn set_face_nominal_angles(&mut self, face_idx: FaceIndex, nominal_angles: Vector3F);

    fn set_node_crease(&mut self, node_crease_idx: NodeCreaseIndex, spec: NodeCreaseSpec);
    fn set_node_beam(&mut self, node_beam_idx: NodeBeamIndex, spec: NodeBeamSpec);
    fn set_node_face(&mut self, node_face_idx: NodeFaceIndex, spec: NodeFaceSpec);
}

static_assertions::assert_obj_safe!(Destination);

pub trait MappedDestination<'container> {
    type NodePositionMap<'a>: DerefMut<Target = [Vector3F]> + 'a
    where
        Self: 'a,
        'a: 'container;
    fn map_node_position<'call>(&'call mut self) -> Self::NodePositionMap<'call>
    where
        'call: 'container;

    type NodeExternalForcesMap<'a>: DerefMut<Target = [Vector3F]> + 'a
    where
        Self: 'a,
        'a: 'container;
    fn map_node_external_forces<'call>(&'call mut self) -> Self::NodeExternalForcesMap<'call>
    where
        'call: 'container;

    type NodeConfigMap<'a>: DerefMut<Target = [NodeConfig]> + 'a
    where
        Self: 'a,
        'a: 'container;
    fn map_node_config<'call>(&'call mut self) -> Self::NodeConfigMap<'call>
    where
        'call: 'container;

    type NodeGeometryMap<'a>: DerefMut<Target = [NodeGeometry]> + 'a
    where
        Self: 'a,
        'a: 'container;
    fn map_node_geometry<'call>(&'call mut self) -> Self::NodeGeometryMap<'call>
    where
        'call: 'container;

    type CreaseGeometryMap<'a>: DerefMut<Target = [CreaseGeometry]> + 'a
    where
        Self: 'a,
        'a: 'container;
    fn map_crease_geometry<'call>(&'call mut self) -> Self::CreaseGeometryMap<'call>
    where
        'call: 'container;

    type CreaseParametersMap<'a>: DerefMut<Target = [CreaseParameters]> + 'a
    where
        Self: 'a,
        'a: 'container;
    fn map_crease_parameters<'call>(&'call mut self) -> Self::CreaseParametersMap<'call>
    where
        'call: 'container;

    type FaceIndicesMap<'a>: DerefMut<Target = [Vector3U]> + 'a
    where
        Self: 'a,
        'a: 'container;
    fn map_face_indices<'call>(&'call mut self) -> Self::FaceIndicesMap<'call>
    where
        'call: 'container;

    type FaceNominalAnglesMap<'a>: DerefMut<Target = [Vector3F]> + 'a
    where
        Self: 'a,
        'a: 'container;
    fn map_face_nominal_angles<'call>(&'call mut self) -> Self::FaceNominalAnglesMap<'call>
    where
        'call: 'container;

    type NodeCreaseMap<'a>: DerefMut<Target = [NodeCreaseSpec]> + 'a
    where
        Self: 'a,
        'a: 'container;
    fn map_node_crease<'call>(&'call mut self) -> Self::NodeCreaseMap<'call>
    where
        'call: 'container;

    type NodeBeamMap<'a>: DerefMut<Target = [NodeBeamSpec]> + 'a
    where
        Self: 'a,
        'a: 'container;
    fn map_node_beam<'call>(&'call mut self) -> Self::NodeBeamMap<'call>
    where
        'call: 'container;

    type NodeFaceMap<'a>: DerefMut<Target = [NodeFaceSpec]> + 'a
    where
        Self: 'a,
        'a: 'container;
    fn map_node_face<'call>(&'call mut self) -> Self::NodeFaceMap<'call>
    where
        'call: 'container;
}

pub trait MappedResults<'container> {
    type NodePositionOffsetMap<'a>: Deref<Target = [Vector3F]> + 'a
    where
        Self: 'a,
        'a: 'container;
    fn map_node_position_offset<'call>(&'call self) -> Option<Self::NodePositionOffsetMap<'call>>
    where
        'call: 'container;

    type NodeErrorMap<'a>: Deref<Target = [f32]> + 'a
    where
        Self: 'a,
        'a: 'container;
    fn map_node_error<'call>(&'call self) -> Option<Self::NodeErrorMap<'call>>
    where
        'call: 'container;
}

#[cfg(feature = "define_proxy")]
mod proxy_sa {
    use crate::proxy::*;
    use static_assertions as sa;

    macro_rules! check_proxy{
        ($t:ty) => {
            sa::assert_impl_all!(
                Proxy<&'_ mut [u8], $t>: core::ops::DerefMut<Target=[$t]>
            );
        }
    }
    check_proxy!(crate::Vector3F);
    check_proxy!(crate::Vector3U);
    check_proxy!(crate::Vector2U);
    check_proxy!(crate::NodeConfig);
    check_proxy!(crate::NodeGeometry);
    check_proxy!(crate::CreaseGeometry);
    check_proxy!(crate::CreaseParameters);
    check_proxy!(crate::NodeCreaseSpec);
    check_proxy!(crate::NodeBeamSpec);
    check_proxy!(crate::NodeFaceSpec);
}
