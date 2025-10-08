use super::{FrameCore, InheritingFrame, NonKeyFrame};
use crate::{collections, Frame, FrameIndex};
use alloc::borrow::Cow;

#[derive(Debug, Clone, Copy)]
pub enum FrameRef<'a, Alloc>
where
    Alloc: core::alloc::Allocator,
{
    Key(&'a FrameCore<Alloc>),
    NonInheriting {
        core: &'a FrameCore<Alloc>,
        parent: Option<u16>,
    },
    Inheriting(InheritingFrame<'a, Alloc>),
}

impl<'a, Alloc> FrameRef<'a, Alloc>
where
    Alloc: core::alloc::Allocator,
{
    pub fn create(
        frames: &'a [NonKeyFrame<Alloc>],
        key_frame: &'a FrameCore<Alloc>,
        frame_index: FrameIndex,
    ) -> Option<Self> {
        if frame_index == 0 {
            return Some(FrameRef::Key(key_frame));
        }

        let referred = frames.get(usize::from(frame_index - 1));

        match referred {
            None => None, // Referring to non-existent frame
            Some(NonKeyFrame {
                inherit: None | Some(false),
                parent,
                frame: core,
                ..
            }) => Some(Self::NonInheriting {
                core,
                parent: *parent,
            }),
            Some(NonKeyFrame {
                inherit: Some(true),
                ..
            }) => Some(Self::Inheriting(InheritingFrame {
                frames,
                key_frame,
                frame_index,
            })),
        }
    }

    /// To get a core frame, allocations may be needed to resolve the whole
    /// parenting/inheritance logic
    pub fn resolve_in(&'a self, _allocator: Alloc) -> Cow<'a, FrameCore<Alloc>>
    where
        Alloc: Clone,
    {
        match self {
            Self::Key(core) | Self::NonInheriting { core, .. } => Cow::Borrowed(core),
            Self::Inheriting(_child) => todo!("use the allocator to manifest a clone"),
        }
    }
}

macro_rules! apply_dispatch {
    ($kind:ident, fn $name:ident(&self) -> $rv:ty) => {
        fn $name(&self) -> $rv {
            match self.0 {
                FrameRef::Key(core) | FrameRef::NonInheriting { core, .. } => core.$kind().$name(),
                FrameRef::Inheriting(child) => child.$kind().$name(),
            }
        }
    };
}

use crate::collections::NUSlice;

pub struct FrameRefVertices<'a, Alloc>(&'a FrameRef<'a, Alloc>)
where
    Alloc: core::alloc::Allocator;

impl<'a, A> crate::FrameVertices<'a> for FrameRefVertices<'a, A>
where
    A: core::alloc::Allocator,
{
    apply_dispatch!(vertices, fn count(&self) -> usize);

    apply_dispatch!(vertices, fn coords(&self) -> NUSlice<'a, f32>);
    apply_dispatch!(vertices, fn adjacent(&self) -> NUSlice<'a, crate::VertexIndex>);
    apply_dispatch!(vertices, fn edges(&self) -> NUSlice<'a, crate::EdgeIndex>);
    apply_dispatch!(vertices, fn faces(&self) -> NUSlice<'a, crate::MaskableFaceIndex>);
}

pub struct FrameRefEdges<'a, Alloc>(&'a FrameRef<'a, Alloc>)
where
    Alloc: core::alloc::Allocator;

impl<'a, Alloc> crate::FrameEdges<'a> for FrameRefEdges<'a, Alloc>
where
    Alloc: core::alloc::Allocator,
{
    apply_dispatch!(edges, fn count(&self) -> usize);

    apply_dispatch!(edges, fn vertices(&self) -> &'a [crate::EdgeVertexIndices]);
    apply_dispatch!(edges, fn faces(&self) -> NUSlice<'a, crate::MaskableVertexIndex>);
    apply_dispatch!(edges, fn assignment(&self) -> &'a [crate::EdgeAssignment]);
    apply_dispatch!(edges, fn fold_angle(&self) -> &'a [crate::collections::MaskableFloat]);
    apply_dispatch!(edges, fn length(&self) -> &'a [f32]);
}

pub struct FrameRefFaces<'a, Alloc>(&'a FrameRef<'a, Alloc>)
where
    Alloc: core::alloc::Allocator;

impl<'a, Alloc> crate::FrameFaces<'a> for FrameRefFaces<'a, Alloc>
where
    Alloc: core::alloc::Allocator,
{
    apply_dispatch!(faces, fn count(&self) -> usize);

    apply_dispatch!(faces, fn vertices(&self) -> NUSlice<'a, crate::VertexIndex>);
    apply_dispatch!(faces, fn edges(&self) -> NUSlice<'a, crate::EdgeIndex>);
    apply_dispatch!(faces, fn uvs(&self) -> NUSlice<'a, u32>);
}

impl<'a, Alloc> Frame<'a> for &'a FrameRef<'a, Alloc>
where
    Alloc: core::alloc::Allocator,
{
    type Vertices = FrameRefVertices<'a, Alloc>;
    type Edges = FrameRefEdges<'a, Alloc>;
    type Faces = FrameRefFaces<'a, Alloc>;

    fn vertices(&self) -> Self::Vertices {
        FrameRefVertices(self)
    }

    fn edges(&self) -> Self::Edges {
        FrameRefEdges(self)
    }

    fn faces(&self) -> Self::Faces {
        FrameRefFaces(self)
    }
}
