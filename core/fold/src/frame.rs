use alloc::borrow::Cow;

use crate::collections::{Lockstep, SeededOption, String};

use super::*;

#[derive(serde_seeded::DeserializeSeeded, Debug, Clone, serde::Serialize)]
#[seeded(de(seed(crate::deser::Seed<'alloc>)))]
pub struct FrameMetadata<'alloc> {
    #[serde(rename = "frame_title")]
    pub title: SeededOption<String<'alloc>>,

    #[serde(rename = "frame_description")]
    pub description: SeededOption<String<'alloc>>,

    #[serde(rename = "frame_classes")]
    pub classes: Lockstep<'alloc, String<'alloc>>,

    #[serde(rename = "frame_attributes")]
    pub attributes: Lockstep<'alloc, String<'alloc>>,

    #[serde(rename = "frame_unit")]
    pub unit: SeededOption<String<'alloc>>,
}

pub trait FrameVertices<'a> {
    fn count(&self) -> usize {0}
    
    fn coords(&self) -> &'a crate::collections::LockstepNU<'a, f32> {&SeededOption(None)} 
    fn adjacent(&self) -> &'a crate::collections::LockstepNU<'a, VertexIndex> {&SeededOption(None)}
    fn edges(&self) -> &'a crate::collections::LockstepNU<'a, EdgeIndex> {&SeededOption(None)}
    fn faces(&self) -> &'a crate::collections::LockstepNU<'a, Option<FaceIndex>> {&SeededOption(None)}
}

pub trait FrameEdges<'a> {
    fn count(&self) -> usize {0}
    
    fn vertices(&self) -> &'a crate::collections::Lockstep<'a, EdgeVertexIndices> {&SeededOption(None)}
    fn faces(&self) -> &'a crate::collections::LockstepNU<'a, Option<VertexIndex>> {&SeededOption(None)}
    fn assignment(&self) -> &'a crate::collections::Lockstep<'a, EdgeAssignment> {&SeededOption(None)}
    fn fold_angle(&self) -> &'a crate::collections::Lockstep<'a, Option<f32>> {&SeededOption(None)}
    fn length(&self) -> &'a crate::collections::Lockstep<'a, f32> {&SeededOption(None)}
}

pub trait FrameFaces<'a> {
    fn count(&self) -> usize {0}
    
    fn vertices(&self) -> &'a crate::collections::LockstepNU<'a, VertexIndex> {&SeededOption(None)}
    fn edges(&self) -> &'a crate::collections::LockstepNU<'a, EdgeIndex> {&SeededOption(None)}
    fn uvs(&self) -> &'a crate::collections::LockstepNU<'a, u32> {&SeededOption(None)}
}

pub trait Frame<'a> {
    type Vertices: FrameVertices<'a>;
    type Edges: FrameEdges<'a>;
    type Faces: FrameFaces<'a>;

    fn vertices(&self) -> Self::Vertices;
    fn edges(&self) -> Self::Edges;
    fn faces(&self) -> Self::Faces;
}

#[derive(serde_seeded::DeserializeSeeded, Debug, Clone, serde::Serialize)]
#[seeded(de(seed(crate::deser::Seed<'alloc>)))]
pub struct FrameCore<'alloc> {
    #[serde(flatten)]
    pub metadata: FrameMetadata<'alloc>,

    #[serde(flatten)]
    pub vertices: VertexInformation<'alloc>,

    #[serde(flatten)]
    pub edges: EdgeInformation<'alloc>,

    #[serde(flatten)]
    pub faces: FaceInformation<'alloc>,

    #[serde(flatten)]
    pub layering: LayerInformation<'alloc>,

    pub uvs: Lockstep<'alloc, [f32; 2]>,
}

impl<'a> Frame<'a> for &'a FrameCore<'a> {
    type Vertices = &'a VertexInformation<'a>;
    type Edges = &'a EdgeInformation<'a>;
    type Faces = &'a FaceInformation<'a>;

    fn vertices(&self) -> Self::Vertices {
        &self.vertices
    }

    fn edges(&self) -> Self::Edges {
        &self.edges
    }

    fn faces(&self) -> Self::Faces {
        &self.faces
    }
}

#[derive(serde_seeded::DeserializeSeeded, Debug, Clone, serde::Serialize)]
#[seeded(de(seed(crate::deser::Seed<'alloc>)))]
pub struct NonKeyFrame<'alloc> {
    #[serde(flatten)]
    pub frame: FrameCore<'alloc>,
    #[serde(rename = "frame_parent")]
    pub parent: Option<FrameIndex>,
    #[serde(rename = "frame_inherit")]
    pub inherit: Option<bool>,
}



#[derive(Debug, Clone, Copy)]
pub struct InheritingFrame<'a> {
    frames: &'a [NonKeyFrame<'a>],
    key_frame: &'a FrameCore<'a>,
    frame_index: FrameIndex
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

impl<'a> FrameVertices<'a> for InheritingFrameVertices<'a> {
    apply_inheritance!(vertices, fn count(&self) -> usize, 0);
    
    apply_inheritance!(vertices, fn coords(&self) -> &'a crate::collections::LockstepNU<'a, f32>, SeededOption(None));
    apply_inheritance!(vertices, fn adjacent(&self) -> &'a crate::collections::LockstepNU<'a, VertexIndex>, SeededOption(None));
    apply_inheritance!(vertices, fn edges(&self) -> &'a crate::collections::LockstepNU<'a, EdgeIndex>, SeededOption(None));
    apply_inheritance!(vertices, fn faces(&self) -> &'a crate::collections::LockstepNU<'a, Option<FaceIndex>>, SeededOption(None));
}

pub struct InheritingFrameEdges<'a>(&'a InheritingFrame<'a>);

impl<'a> FrameEdges<'a> for InheritingFrameEdges<'a> {
    apply_inheritance!(edges, fn count(&self) -> usize, 0);
    
    apply_inheritance!(edges, fn vertices(&self) -> &'a crate::collections::Lockstep<'a, EdgeVertexIndices>, &SeededOption(None));
    apply_inheritance!(edges, fn faces(&self) -> &'a crate::collections::LockstepNU<'a, Option<VertexIndex>>, &SeededOption(None));
    apply_inheritance!(edges, fn assignment(&self) -> &'a crate::collections::Lockstep<'a, EdgeAssignment>, &SeededOption(None));
    apply_inheritance!(edges, fn fold_angle(&self) -> &'a crate::collections::Lockstep<'a, Option<f32>>, &SeededOption(None));
    apply_inheritance!(edges, fn length(&self) -> &'a crate::collections::Lockstep<'a, f32>, &SeededOption(None));
}

pub struct InheritingFrameFaces<'a>(&'a InheritingFrame<'a>);

impl<'a> FrameFaces<'a> for InheritingFrameFaces<'a> {
    apply_inheritance!(faces, fn count(&self) -> usize, 0);
    
    apply_inheritance!(faces, fn vertices(&self) -> &'a crate::collections::LockstepNU<'a, VertexIndex>, &SeededOption(None));
    apply_inheritance!(faces, fn edges(&self) -> &'a crate::collections::LockstepNU<'a, EdgeIndex>, &SeededOption(None));
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
    fn itself(&self) -> &'a NonKeyFrame<'a> {
        &self.frames[usize::from(self.frame_index - 1)]
    }

    fn parent(&self) -> FrameRef<'a> {
        let parent = self.itself().parent.unwrap();
        FrameRef::create(self.frames, self.key_frame, parent).expect("inheritence at this point should be well-specified")
    }
}

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

impl<'a> FrameVertices<'a> for FrameRefVertices<'a> {
    apply_dispatch!(vertices, fn count(&self) -> usize, 0);
    
    apply_dispatch!(vertices, fn coords(&self) -> &'a crate::collections::LockstepNU<'a, f32>, SeededOption(None));
    apply_dispatch!(vertices, fn adjacent(&self) -> &'a crate::collections::LockstepNU<'a, VertexIndex>, SeededOption(None));
    apply_dispatch!(vertices, fn edges(&self) -> &'a crate::collections::LockstepNU<'a, EdgeIndex>, SeededOption(None));
    apply_dispatch!(vertices, fn faces(&self) -> &'a crate::collections::LockstepNU<'a, Option<FaceIndex>>, SeededOption(None));
}

pub struct FrameRefEdges<'a>(&'a FrameRef<'a>);

impl<'a> FrameEdges<'a> for FrameRefEdges<'a> {
    apply_dispatch!(edges, fn count(&self) -> usize, 0);
    
    apply_dispatch!(edges, fn vertices(&self) -> &'a crate::collections::Lockstep<'a, EdgeVertexIndices>, &SeededOption(None));
    apply_dispatch!(edges, fn faces(&self) -> &'a crate::collections::LockstepNU<'a, Option<VertexIndex>>, &SeededOption(None));
    apply_dispatch!(edges, fn assignment(&self) -> &'a crate::collections::Lockstep<'a, EdgeAssignment>, &SeededOption(None));
    apply_dispatch!(edges, fn fold_angle(&self) -> &'a crate::collections::Lockstep<'a, Option<f32>>, &SeededOption(None));
    apply_dispatch!(edges, fn length(&self) -> &'a crate::collections::Lockstep<'a, f32>, &SeededOption(None));
}

pub struct FrameRefFaces<'a>(&'a FrameRef<'a>);

impl<'a> FrameFaces<'a> for FrameRefFaces<'a> {
    apply_dispatch!(faces, fn count(&self) -> usize, 0);
    
    apply_dispatch!(faces, fn vertices(&self) -> &'a crate::collections::LockstepNU<'a, VertexIndex>, &SeededOption(None));
    apply_dispatch!(faces, fn edges(&self) -> &'a crate::collections::LockstepNU<'a, EdgeIndex>, &SeededOption(None));
    apply_dispatch!(faces, fn uvs(&self) -> &'a crate::collections::LockstepNU<'a, u32>, &SeededOption(None));
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
