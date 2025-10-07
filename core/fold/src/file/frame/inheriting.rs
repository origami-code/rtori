use super::{NonKeyFrame, FrameCore, FrameRef};
use crate::{FrameIndex, Frame};
use crate::collections::SeededOption;

#[derive(Debug, Clone, Copy)]
pub struct InheritingFrame<'a> {
    pub(super) frames: &'a [NonKeyFrame<'a>],
    pub(super) key_frame: &'a FrameCore<'a>,
    pub(super) frame_index: FrameIndex
}

macro_rules! apply_inheritance {
    ($kind:ident, fn $name:ident(&self) -> $rv:ty, $default:pat) => {
        fn $name(&self) -> $rv {
            let overriden = self.0.key_frame.$kind().$name();
            match overriden {
                $default => todo!("implement a way for the FrameRef to be borrowed"), // self.0.parent().$kind().$name(),
                other => other
            }
        }
    }
}

pub struct InheritingFrameVertices<'a>(&'a InheritingFrame<'a>);

impl<'a> crate::FrameVertices<'a> for InheritingFrameVertices<'a> {
    apply_inheritance!(vertices, fn count(&self) -> usize, 0);
    
    apply_inheritance!(vertices, fn coords(&self) -> &'a crate::collections::LockstepNU<'a, f32>, SeededOption(None));
    apply_inheritance!(vertices, fn adjacent(&self) -> &'a crate::collections::LockstepNU<'a, crate::VertexIndex>, SeededOption(None));
    apply_inheritance!(vertices, fn edges(&self) -> &'a crate::collections::LockstepNU<'a, crate::EdgeIndex>, SeededOption(None));
    apply_inheritance!(vertices, fn faces(&self) -> &'a crate::collections::LockstepNU<'a, Option<crate::FaceIndex>>, SeededOption(None));
}

pub struct InheritingFrameEdges<'a>(&'a InheritingFrame<'a>);

impl<'a> crate::FrameEdges<'a> for InheritingFrameEdges<'a> {
    apply_inheritance!(edges, fn count(&self) -> usize, 0);
    
    apply_inheritance!(edges, fn vertices(&self) -> &'a crate::collections::Lockstep<'a, crate::EdgeVertexIndices>, &SeededOption(None));
    apply_inheritance!(edges, fn faces(&self) -> &'a crate::collections::LockstepNU<'a, Option<crate::VertexIndex>>, &SeededOption(None));
    apply_inheritance!(edges, fn assignment(&self) -> &'a crate::collections::Lockstep<'a, crate::EdgeAssignment>, &SeededOption(None));
    apply_inheritance!(edges, fn fold_angle(&self) -> &'a crate::collections::Lockstep<'a, Option<f32>>, &SeededOption(None));
    apply_inheritance!(edges, fn length(&self) -> &'a crate::collections::Lockstep<'a, f32>, &SeededOption(None));
}

pub struct InheritingFrameFaces<'a>(&'a InheritingFrame<'a>);

impl<'a> crate::FrameFaces<'a> for InheritingFrameFaces<'a> {
    apply_inheritance!(faces, fn count(&self) -> usize, 0);
    
    apply_inheritance!(faces, fn vertices(&self) -> &'a crate::collections::LockstepNU<'a, crate::VertexIndex>, &SeededOption(None));
    apply_inheritance!(faces, fn edges(&self) -> &'a crate::collections::LockstepNU<'a, crate::EdgeIndex>, &SeededOption(None));
    apply_inheritance!(faces, fn uvs(&self) -> &'a crate::collections::LockstepNU<'a, u32>, &SeededOption(None));
}

impl<'a> Frame<'a> for &'a InheritingFrame<'a> {
    type Vertices = InheritingFrameVertices<'a>;
    type Edges = InheritingFrameEdges<'a>;
    type Faces = InheritingFrameFaces<'a>;

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

impl<'a> InheritingFrame<'a> {
    pub fn itself(&self) -> &'a NonKeyFrame<'a> {
        &self.frames[usize::from(self.frame_index - 1)]
    }

    pub fn parent(&self) -> FrameRef<'a> {
        let parent = self.itself().parent.unwrap();
        FrameRef::create(self.frames, self.key_frame, parent).expect("inheritence at this point should be well-specified")
    }
}