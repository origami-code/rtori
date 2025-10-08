use super::{FrameCore, FrameRef, NonKeyFrame};
use crate::collections::{NUSlice, SeededOption};
use crate::{Frame, FrameIndex, MaskableFaceIndex, MaskableVertexIndex};

#[derive(Debug, Clone, Copy)]
pub struct InheritingFrame<'a, A>
where
    A: core::alloc::Allocator,
{
    pub(super) frames: &'a [NonKeyFrame<A>],
    pub(super) key_frame: &'a FrameCore<A>,
    pub(super) frame_index: FrameIndex,
}

macro_rules! apply_inheritance_default {
    ($kind:ident, fn $name:ident(&self) -> $rv:ty) => {
        fn $name(&self) -> $rv {
            let overriden = self.0.key_frame.$kind().$name();
            if overriden == <$rv>::default() {
                (&self.0.parent()).$kind().$name()
            } else {
                overriden
            }
        }
    };
}

macro_rules! apply_inheritance {
    ($kind:ident, fn $name:ident(&self) -> $rv:ty) => {
        fn $name(&self) -> $rv {
            let overriden = self.0.key_frame.$kind().$name();
            if overriden.len() == 0 {
                // self.0.parent().$kind().$name(),
                todo!("implement a way for the FrameRef to be borrowed")
            }
            overriden
        }
    };
}

pub struct InheritingFrameVertices<'a, A>(&'a InheritingFrame<'a, A>)
where
    A: core::alloc::Allocator;

impl<'a, A> crate::FrameVertices<'a> for InheritingFrameVertices<'a, A>
where
    A: core::alloc::Allocator,
{
    apply_inheritance_default!(vertices, fn count(&self) -> usize);

    apply_inheritance!(vertices, fn coords(&self) -> NUSlice<'a, f32>);
    apply_inheritance!(vertices, fn adjacent(&self) -> NUSlice<'a, crate::VertexIndex>);
    apply_inheritance!(vertices, fn edges(&self) -> NUSlice<'a, crate::EdgeIndex>);
    apply_inheritance!(vertices, fn faces(&self) -> NUSlice<'a, MaskableFaceIndex>);
}

pub struct InheritingFrameEdges<'a, A>(&'a InheritingFrame<'a, A>)
where
    A: core::alloc::Allocator;

impl<'a, A> crate::FrameEdges<'a> for InheritingFrameEdges<'a, A>
where
    A: core::alloc::Allocator,
{
    apply_inheritance_default!(edges, fn count(&self) -> usize);

    apply_inheritance!(edges, fn vertices(&self) -> &'a [crate::EdgeVertexIndices]);
    apply_inheritance!(edges, fn faces(&self) -> NUSlice<'a, MaskableVertexIndex>);
    apply_inheritance!(edges, fn assignment(&self) -> &'a [crate::EdgeAssignment]);
    apply_inheritance!(edges, fn fold_angle(&self) -> &'a [crate::collections::MaskableFloat]);
    apply_inheritance!(edges, fn length(&self) -> &'a [f32]);
}

pub struct InheritingFrameFaces<'a, A>(&'a InheritingFrame<'a, A>)
where
    A: core::alloc::Allocator;

impl<'a, A> crate::FrameFaces<'a> for InheritingFrameFaces<'a, A>
where
    A: core::alloc::Allocator,
{
    apply_inheritance_default!(faces, fn count(&self) -> usize);

    apply_inheritance!(faces, fn vertices(&self) -> NUSlice<'a, crate::VertexIndex>);
    apply_inheritance!(faces, fn edges(&self) -> NUSlice<'a, crate::EdgeIndex>);
    apply_inheritance!(faces, fn uvs(&self) -> NUSlice<'a, u32>);
}

impl<'a, A> Frame<'a> for &'a InheritingFrame<'a, A>
where
    A: core::alloc::Allocator,
{
    type Vertices = InheritingFrameVertices<'a, A>;
    type Edges = InheritingFrameEdges<'a, A>;
    type Faces = InheritingFrameFaces<'a, A>;

    fn vertices(&self) -> Self::Vertices {
        InheritingFrameVertices(self)
    }

    fn edges(&self) -> Self::Edges {
        InheritingFrameEdges(self)
    }

    fn faces(&self) -> Self::Faces {
        InheritingFrameFaces(self)
    }
}

impl<'a, A> InheritingFrame<'a, A>
where
    A: core::alloc::Allocator,
{
    pub fn itself(&self) -> &'a NonKeyFrame<A> {
        &self.frames[usize::from(self.frame_index - 1)]
    }

    pub fn parent(&self) -> FrameRef<'a, A> {
        let parent = self.itself().parent.unwrap();
        FrameRef::create(self.frames, self.key_frame, parent)
            .expect("inheritence at this point should be well-specified")
    }
}
