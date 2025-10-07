use alloc::borrow::Cow;
use super::{FrameCore, InheritingFrame, NonKeyFrame};
use crate::{FrameIndex, Frame, collections};

#[derive(Debug, Clone, Copy)]
pub enum FrameRef<'a> {
    Key(&'a FrameCore<'a>),
    NonInheriting{core: &'a FrameCore<'a>, parent: Option<u16>},
    Inheriting(InheritingFrame<'a>)
}

impl<'a> FrameRef<'a> {
    pub fn create(frames: &'a [NonKeyFrame<'a>], key_frame: &'a FrameCore<'a>, frame_index: FrameIndex) -> Option<Self> {
        if frame_index == 0 {
            return Some(FrameRef::Key(key_frame));
        } 

        let referred =  frames
            .get(usize::from(frame_index - 1));
        
        match referred {
            None => None, // Referring to non-existent frame
            Some(NonKeyFrame{inherit: None | Some(false), parent, frame: core, ..}) => Some(Self::NonInheriting{core, parent: *parent}),
            Some(NonKeyFrame{inherit: Some(true), ..}) => Some(Self::Inheriting(
                InheritingFrame { frames, key_frame, frame_index }
            ))
        }
    }

    /// To get a core frame, allocations may be needed to resolve the whole
    /// parenting/inheritance logic
    pub fn resolve(&'a self) -> Cow<'a, FrameCore> {
        match self {
            Self::Key(core) | Self::NonInheriting { core, .. } => Cow::Borrowed(core),
            Self::Inheriting(_child) => unimplemented!(),
        }
    }
}


macro_rules! apply_dispatch {
    ($kind:ident, fn $name:ident(&self) -> $rv:ty, $default:pat) => {
        fn $name(&self) -> $rv {
            match self.0 {
                FrameRef::Key(core) | FrameRef::NonInheriting { core, .. } => core.$kind().$name(),
                FrameRef::Inheriting(child) => child.$kind().$name(),
            }
        }
    }
}

pub struct FrameRefVertices<'a>(&'a FrameRef<'a>);

impl<'a> crate::FrameVertices<'a> for FrameRefVertices<'a> {
    apply_dispatch!(vertices, fn count(&self) -> usize, 0);
    
    apply_dispatch!(vertices, fn coords(&self) -> &'a collections::LockstepNU<'a, f32>, SeededOption(None));
    apply_dispatch!(vertices, fn adjacent(&self) -> &'a collections::LockstepNU<'a, crate::VertexIndex>, SeededOption(None));
    apply_dispatch!(vertices, fn edges(&self) -> &'a collections::LockstepNU<'a, crate::EdgeIndex>, SeededOption(None));
    apply_dispatch!(vertices, fn faces(&self) -> &'a collections::LockstepNU<'a, Option<crate::FaceIndex>>, SeededOption(None));
}

pub struct FrameRefEdges<'a>(&'a FrameRef<'a>);

impl<'a> crate::FrameEdges<'a> for FrameRefEdges<'a> {
    apply_dispatch!(edges, fn count(&self) -> usize, 0);
    
    apply_dispatch!(edges, fn vertices(&self) -> &'a collections::Lockstep<'a, crate::EdgeVertexIndices>, &SeededOption(None));
    apply_dispatch!(edges, fn faces(&self) -> &'a collections::LockstepNU<'a, Option<crate::VertexIndex>>, &SeededOption(None));
    apply_dispatch!(edges, fn assignment(&self) -> &'a collections::Lockstep<'a, crate::EdgeAssignment>, &SeededOption(None));
    apply_dispatch!(edges, fn fold_angle(&self) -> &'a collections::Lockstep<'a, Option<f32>>, &SeededOption(None));
    apply_dispatch!(edges, fn length(&self) -> &'a collections::Lockstep<'a, f32>, &SeededOption(None));
}

pub struct FrameRefFaces<'a>(&'a FrameRef<'a>);

impl<'a> crate::FrameFaces<'a> for FrameRefFaces<'a> {
    apply_dispatch!(faces, fn count(&self) -> usize, 0);
    
    apply_dispatch!(faces, fn vertices(&self) -> &'a collections::LockstepNU<'a, crate::VertexIndex>, &SeededOption(None));
    apply_dispatch!(faces, fn edges(&self) -> &'a collections::LockstepNU<'a, crate::EdgeIndex>, &SeededOption(None));
    apply_dispatch!(faces, fn uvs(&self) -> &'a collections::LockstepNU<'a, u32>, &SeededOption(None));
}

impl<'a> Frame<'a> for &'a FrameRef<'a> {
    type Vertices = FrameRefVertices<'a>;
    type Edges = FrameRefEdges<'a>;
    type Faces = FrameRefFaces<'a>;

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