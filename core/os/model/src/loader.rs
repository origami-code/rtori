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
        'container: 'a;
    fn access_node_position<'call, 'output>(&'call mut self) -> Self::NodePositionAccess<'output>
    where
        'call: 'output,
        'container: 'output;

    type NodeExternalForcesAccess<'a>: WriteAccess<'a, Vector3F> + 'a
    where
        Self: 'a,
        'container: 'a;
    fn access_node_external_forces<'call, 'output>(
        &'call mut self,
    ) -> Self::NodeExternalForcesAccess<'output>
    where
        'call: 'output,
        'container: 'output;

    type NodeConfigAccess<'a>: WriteAccess<'a, NodeConfig> + 'a
    where
        Self: 'a,
        'container: 'a;
    fn access_node_config<'call, 'output>(&'call mut self) -> Self::NodeConfigAccess<'output>
    where
        'call: 'output,
        'container: 'output;

    type NodeGeometryAccess<'a>: WriteAccess<'a, NodeGeometry> + 'a
    where
        Self: 'a,
        'container: 'a;
    fn access_node_geometry<'call, 'output>(&'call mut self) -> Self::NodeGeometryAccess<'output>
    where
        'call: 'output,
        'container: 'output;

    type CreaseGeometryAccess<'a>: WriteAccess<'a, CreaseGeometry> + 'a
    where
        Self: 'a,
        'container: 'a;
    fn access_crease_geometry<'call, 'output>(
        &'call mut self,
    ) -> Self::CreaseGeometryAccess<'output>
    where
        'call: 'output,
        'container: 'output;

    type CreaseParametersAccess<'a>: WriteAccess<'a, CreaseParameters> + 'a
    where
        Self: 'a,
        'container: 'a;
    fn access_crease_parameters<'call, 'output>(
        &'call mut self,
    ) -> Self::CreaseParametersAccess<'output>
    where
        'call: 'output,
        'container: 'output;

    type FaceIndicesAccess<'a>: WriteAccess<'a, Vector3U> + 'a
    where
        Self: 'a,
        'container: 'a;
    fn access_face_indices<'call, 'output>(&'call mut self) -> Self::FaceIndicesAccess<'output>
    where
        'call: 'output,
        'container: 'output;

    type FaceNominalAnglesAccess<'a>: WriteAccess<'a, Vector3F> + 'a
    where
        Self: 'a,
        'container: 'a;
    fn access_face_nominal_angles<'call, 'output>(
        &'call mut self,
    ) -> Self::FaceNominalAnglesAccess<'output>
    where
        'call: 'output,
        'container: 'output;

    type NodeCreaseAccess<'a>: WriteAccess<'a, NodeCreaseSpec> + 'a
    where
        Self: 'a,
        'container: 'a;
    fn access_node_crease<'call, 'output>(&'call mut self) -> Self::NodeCreaseAccess<'output>
    where
        'call: 'output,
        'container: 'output;

    type NodeBeamAccess<'a>: WriteAccess<'a, NodeBeamSpec> + 'a
    where
        Self: 'a,
        'container: 'a;
    fn access_node_beam<'call, 'output>(&'call mut self) -> Self::NodeBeamAccess<'output>
    where
        'call: 'output,
        'container: 'output;

    type NodeFaceAccess<'a>: WriteAccess<'a, NodeFaceSpec> + 'a
    where
        Self: 'a,
        'container: 'a;
    fn access_node_face<'call, 'output>(&'call mut self) -> Self::NodeFaceAccess<'output>
    where
        'call: 'output,
        'container: 'output;
}

pub trait LoaderDyn<'container> {
    fn model(&self) -> crate::ModelSize;
    fn copy_node_position(&mut self, from: &[Vector3F], offset: NodeIndex);
    fn copy_node_external_forces(&mut self, from: &[Vector3F], offset: NodeIndex);
    fn copy_node_config(&mut self, from: &[NodeConfig], offset: NodeIndex);
    fn copy_node_geometry(&mut self, from: &[NodeGeometry], offset: NodeIndex);
    fn copy_crease_geometry(&mut self, from: &[CreaseGeometry], offset: CreaseIndex);
    fn copy_crease_parameters(&mut self, from: &[CreaseParameters], offset: CreaseIndex);
    fn copy_face_indices(&mut self, from: &[Vector3U], offset: FaceIndex);
    fn copy_face_nominal_angles(&mut self, from: &[Vector3F], offset: FaceIndex);
    fn copy_node_crease(&mut self, from: &[NodeCreaseSpec], offset: NodeCreaseIndex);
    fn copy_node_beam(&mut self, from: &[NodeBeamSpec], offset: NodeBeamIndex);
    fn copy_node_face(&mut self, from: &[NodeFaceSpec], offset: NodeFaceIndex);
}

static_assertions::assert_obj_safe!(LoaderDyn);

impl<'container, Container> LoaderDyn<'container> for Container
where
    Container: Loader<'container>,
{
    fn model(&self) -> crate::ModelSize {
        Loader::model(self)
    }

    fn copy_node_position(&mut self, from: &[Vector3F], offset: NodeIndex) {
        Loader::access_node_position(self).copy_in(from, offset)
    }

    fn copy_node_external_forces(&mut self, from: &[Vector3F], offset: NodeIndex) {
        Loader::access_node_external_forces(self).copy_in(from, offset)
    }

    fn copy_node_config(&mut self, from: &[NodeConfig], offset: NodeIndex) {
        Loader::access_node_config(self).copy_in(from, offset)
    }

    fn copy_node_geometry(&mut self, from: &[NodeGeometry], offset: NodeIndex) {
        Loader::access_node_geometry(self).copy_in(from, offset)
    }

    fn copy_crease_geometry(&mut self, from: &[CreaseGeometry], offset: CreaseIndex) {
        Loader::access_crease_geometry(self).copy_in(from, offset)
    }

    fn copy_crease_parameters(&mut self, from: &[CreaseParameters], offset: CreaseIndex) {
        Loader::access_crease_parameters(self).copy_in(from, offset)
    }

    fn copy_face_indices(&mut self, from: &[Vector3U], offset: FaceIndex) {
        Loader::access_face_indices(self).copy_in(from, offset)
    }

    fn copy_face_nominal_angles(&mut self, from: &[Vector3F], offset: FaceIndex) {
        Loader::access_face_nominal_angles(self).copy_in(from, offset)
    }

    fn copy_node_crease(&mut self, from: &[NodeCreaseSpec], offset: NodeCreaseIndex) {
        Loader::access_node_crease(self).copy_in(from, offset)
    }

    fn copy_node_beam(&mut self, from: &[NodeBeamSpec], offset: NodeBeamIndex) {
        Loader::access_node_beam(self).copy_in(from, offset)
    }

    fn copy_node_face(&mut self, from: &[NodeFaceSpec], offset: NodeFaceIndex) {
        Loader::access_node_face(self).copy_in(from, offset)
    }
}
