use crate::model::*;
use core::ops::DerefMut;

/// Not object safe
pub trait WriteAccess<'container, T> {
    fn capacity(&self) -> usize;
    fn copy_in(&mut self, from: &[T], offset: u32);

    fn set(&mut self, index: u32, value: T) {
        self.copy_in(&[value], index);
    }

    type Mapped<'a>: DerefMut<Target = [T]> + 'a
    where
        Self: 'a,
        'a: 'container;

    fn try_map<'call>(&'call mut self) -> Option<Self::Mapped<'call>>
    where
        'call: 'container,
    {
        None
    }
}

pub trait Loader<'container> {
    fn model(&self) -> crate::ModelSize;

    type NodePositionAccess<'a>: WriteAccess<'a, Vector3F> + 'a
    where
        Self: 'a,
        'a: 'container;
    fn access_node_position<'call>(&'call mut self) -> Self::NodePositionAccess<'call>
    where
        'call: 'container;

    type NodeExternalForcesAccess<'a>: WriteAccess<'a, Vector3F> + 'a
    where
        Self: 'a,
        'a: 'container;
    fn access_node_external_forces<'call>(&'call mut self) -> Self::NodeExternalForcesAccess<'call>
    where
        'call: 'container;

    type NodeConfigAccess<'a>: WriteAccess<'a, NodeConfig> + 'a
    where
        Self: 'a,
        'a: 'container;
    fn access_node_config<'call>(&'call mut self) -> Self::NodeConfigAccess<'call>
    where
        'call: 'container;

    type NodeGeometryAccess<'a>: WriteAccess<'a, NodeGeometry> + 'a
    where
        Self: 'a,
        'a: 'container;

    fn access_node_geometry<'call>(&'call mut self) -> Self::NodeGeometryAccess<'call>
    where
        'call: 'container;

    type CreaseGeometryAccess<'a>: WriteAccess<'a, CreaseGeometry> + 'a
    where
        Self: 'a,
        'a: 'container;
    fn access_crease_geometry<'call>(&'call mut self) -> Self::CreaseGeometryAccess<'call>
    where
        'call: 'container;

    type CreaseParametersAccess<'a>: WriteAccess<'a, CreaseParameters> + 'a
    where
        Self: 'a,
        'a: 'container;
    fn access_crease_parameters<'call>(&'call mut self) -> Self::CreaseParametersAccess<'call>
    where
        'call: 'container;

    type FaceIndicesAccess<'a>: WriteAccess<'a, Vector3U> + 'a
    where
        Self: 'a,
        'a: 'container;
    fn access_face_indices<'call>(&'call mut self) -> Self::FaceIndicesAccess<'call>
    where
        'call: 'container;

    type FaceNominalAnglesAccess<'a>: WriteAccess<'a, Vector3F> + 'a
    where
        Self: 'a,
        'a: 'container;
    fn access_face_nominal_angles<'call>(&'call mut self) -> Self::FaceNominalAnglesAccess<'call>
    where
        'call: 'container;

    type NodeCreaseAccess<'a>: WriteAccess<'a, NodeCreaseSpec> + 'a
    where
        Self: 'a,
        'a: 'container;
    fn access_node_crease<'call>(&'call mut self) -> Self::NodeCreaseAccess<'call>
    where
        'call: 'container;

    type NodeBeamAccess<'a>: WriteAccess<'a, NodeBeamSpec> + 'a
    where
        Self: 'a,
        'a: 'container;
    fn access_node_beam<'call>(&'call mut self) -> Self::NodeBeamAccess<'call>
    where
        'call: 'container;

    type NodeFaceAccess<'a>: WriteAccess<'a, NodeFaceSpec> + 'a
    where
        Self: 'a,
        'a: 'container;
    fn access_node_face<'call>(&'call mut self) -> Self::NodeFaceAccess<'call>
    where
        'call: 'container;
}

pub trait LoaderDyn<'container> {
    fn model(&'container self) -> crate::ModelSize;
    fn copy_node_position(&'container mut self, from: &[Vector3F], offset: NodeIndex);
    fn copy_node_external_forces(&'container mut self, from: &[Vector3F], offset: NodeIndex);
    fn copy_node_config(&'container mut self, from: &[NodeConfig], offset: NodeIndex);
    fn copy_node_geometry(&'container mut self, from: &[NodeGeometry], offset: NodeIndex);
    fn copy_crease_geometry(&'container mut self, from: &[CreaseGeometry], offset: CreaseIndex);
    fn copy_crease_parameters(&'container mut self, from: &[CreaseParameters], offset: CreaseIndex);
    fn copy_face_indices(&'container mut self, from: &[Vector3U], offset: FaceIndex);
    fn copy_face_nominal_angles(&'container mut self, from: &[Vector3F], offset: FaceIndex);
    fn copy_node_crease(&'container mut self, from: &[NodeCreaseSpec], offset: NodeCreaseIndex);
    fn copy_node_beam(&'container mut self, from: &[NodeBeamSpec], offset: NodeBeamIndex);
    fn copy_node_face(&'container mut self, from: &[NodeFaceSpec], offset: NodeFaceIndex);
}

static_assertions::assert_obj_safe!(LoaderDyn);

impl<'container, Container> LoaderDyn<'container> for Container
where
    Container: Loader<'container>,
{
    fn model(&'container self) -> crate::ModelSize {
        Loader::model(self)
    }

    fn copy_node_position(&'container mut self, from: &[Vector3F], offset: NodeIndex) {
        Loader::access_node_position(self).copy_in(from, offset)
    }

    fn copy_node_external_forces(&'container mut self, from: &[Vector3F], offset: NodeIndex) {
        Loader::access_node_external_forces(self).copy_in(from, offset)
    }

    fn copy_node_config(&'container mut self, from: &[NodeConfig], offset: NodeIndex) {
        Loader::access_node_config(self).copy_in(from, offset)
    }

    fn copy_node_geometry(&'container mut self, from: &[NodeGeometry], offset: NodeIndex) {
        Loader::access_node_geometry(self).copy_in(from, offset)
    }

    fn copy_crease_geometry(&'container mut self, from: &[CreaseGeometry], offset: CreaseIndex) {
        Loader::access_crease_geometry(self).copy_in(from, offset)
    }

    fn copy_crease_parameters(
        &'container mut self,
        from: &[CreaseParameters],
        offset: CreaseIndex,
    ) {
        Loader::access_crease_parameters(self).copy_in(from, offset)
    }

    fn copy_face_indices(&'container mut self, from: &[Vector3U], offset: FaceIndex) {
        Loader::access_face_indices(self).copy_in(from, offset)
    }

    fn copy_face_nominal_angles(&'container mut self, from: &[Vector3F], offset: FaceIndex) {
        Loader::access_face_nominal_angles(self).copy_in(from, offset)
    }

    fn copy_node_crease(&'container mut self, from: &[NodeCreaseSpec], offset: NodeCreaseIndex) {
        Loader::access_node_crease(self).copy_in(from, offset)
    }

    fn copy_node_beam(&'container mut self, from: &[NodeBeamSpec], offset: NodeBeamIndex) {
        Loader::access_node_beam(self).copy_in(from, offset)
    }

    fn copy_node_face(&'container mut self, from: &[NodeFaceSpec], offset: NodeFaceIndex) {
        Loader::access_node_face(self).copy_in(from, offset)
    }
}
