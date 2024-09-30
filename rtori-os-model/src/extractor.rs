use crate::model::*;
use core::ops::Deref;

pub trait ReadAccess<'access, T> {
    fn get(&self, index: u32) -> T;

    fn count(&self) -> usize;
    fn copy_out(&self, out: &mut [T], offset: usize);

    type Mapped<'a>: Deref<Target = [T]> + 'a
    where
        Self: 'a,
        'a: 'access;

    fn try_map<'call>(&'call self) -> Option<Self::Mapped<'call>>
    where
        'call: 'access,
    {
        None
    }
}

pub trait Extractor<'container> {
    fn count_nodes(&self) -> usize;

    type NodePositionAccess<'a>: ReadAccess<'a, Vector3F>
    where
        Self: 'a,
        'container: 'a;
    fn access_node_position<'call, 'output>(
        &'call self,
    ) -> Option<Self::NodePositionAccess<'output>>
    where
        'call: 'output,
        'container: 'output;

    type NodeVelocityAccess<'a>: ReadAccess<'a, Vector3F>
    where
        Self: 'a,
        'container: 'a;
    fn access_node_velocity<'call, 'output>(
        &'call self,
    ) -> Option<Self::NodeVelocityAccess<'output>>
    where
        'call: 'output,
        'container: 'output;

    type NodeErrorAccess<'a>: ReadAccess<'a, f32>
    where
        Self: 'a,
        'container: 'a;
    fn access_node_error<'call, 'output>(&'call self) -> Option<Self::NodeErrorAccess<'output>>
    where
        'call: 'output,
        'container: 'output;
}

pub trait ExtractorDyn<'container> {
    fn count_nodes(&self) -> usize;
    fn copy_node_position(&self, to: &mut [Vector3F], from: NodeIndex) -> bool;
    fn copy_node_velocity(&self, to: &mut [Vector3F], from: NodeIndex) -> bool;
    fn copy_node_error(&self, to: &mut [f32], from: NodeIndex) -> bool;
}

static_assertions::assert_obj_safe!(ExtractorDyn);

impl<'container, Container> ExtractorDyn<'container> for Container
where
    Container: Extractor<'container>,
{
    fn count_nodes(&self) -> usize {
        self.count_nodes()
    }

    fn copy_node_position(&self, to: &mut [Vector3F], from: NodeIndex) -> bool {
        self.access_node_position()
            .map(|a| a.copy_out(to, usize::try_from(from).unwrap()))
            .is_some()
    }

    fn copy_node_velocity(&self, to: &mut [Vector3F], from: NodeIndex) -> bool {
        self.access_node_velocity()
            .map(|a| a.copy_out(to, usize::try_from(from).unwrap()))
            .is_some()
    }

    fn copy_node_error(&self, to: &mut [f32], from: NodeIndex) -> bool {
        self.access_node_error()
            .map(|a| a.copy_out(to, usize::try_from(from).unwrap()))
            .is_some()
    }
}
