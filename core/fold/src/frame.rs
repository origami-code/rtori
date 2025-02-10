use alloc::borrow::Cow;

use crate::Lockstep;

use super::*;

#[derive(Debug, Clone /*, serde::Deserialize, serde::Serialize*/)]
pub struct FrameMetadata<'alloc> {
    //#[serde(rename = "frame_title")]
    pub title: Option<String<'alloc>>,

    //#[serde(rename = "frame_description")]
    pub description: Option<String<'alloc>>,

    //#[serde(rename = "frame_classes")]
    pub classes: Option<Vec<'alloc, String<'alloc>>>,

    //#[serde(rename = "frame_attributes")]
    pub attributes: Option<Vec<'alloc, String<'alloc>>>,

    //#[serde(rename = "frame_unit")]
    pub unit: Option<String<'alloc>>,
}

#[derive(Debug, Clone /*, serde::Deserialize, serde::Serialize*/)]
pub struct FrameCore<'alloc> {
    //#[serde(flatten)]
    pub metadata: FrameMetadata<'alloc>,

    //#[serde(flatten)]
    pub vertices: VertexInformation<'alloc>,

    //#[serde(flatten)]
    pub edges: EdgeInformation<'alloc>,

    //#[serde(flatten)]
    pub faces: FaceInformation<'alloc>,

    //#[serde(flatten)]
    pub layering: LayerInformation<'alloc>,

    pub uvs: Lockstep<'alloc, [f32; 2]>,
}

impl<'a> FrameCore<'a> {
    pub fn vertices_count(&self) -> usize {
        self.vertices.count()
    }

    pub fn edges_count(&self) -> usize {
        self.edges.count()
    }

    pub fn faces_count(&self) -> usize {
        self.faces.count()
    }
}

#[derive(Debug, Clone /*, serde::Deserialize, serde::Serialize*/)]
pub struct NonKeyFrame<'alloc> {
    //#[serde(flatten)]
    pub frame: FrameCore<'alloc>,
    //#[serde(rename = "frame_parent")]
    pub parent: Option<FrameIndex>,
    //#[serde(rename = "frame_inherit")]
    pub inherit: Option<bool>,
}

#[derive(Debug, Clone, Copy)]
pub struct InheritingFrame<'a> {
    frames: &'a [NonKeyFrame<'a>],
    key_frame: &'a FrameCore<'a>,
    frame_index: FrameIndex
}

impl<'a> InheritingFrame<'a> {
    fn itself(&self) -> &'a NonKeyFrame {
        &self.frames[usize::from(self.frame_index - 1)]
    }

    fn parent(&self) -> FrameRef<'a> {
        let parent = self.itself().parent.unwrap();
        FrameRef::create(self.frames, self.key_frame, parent).expect("inheritence at this point should be well-specified")
    }

    pub fn vertices_count(&self) -> usize {
        let overriden = self.key_frame.vertices_count();
        match overriden {
            0 => self.parent().vertices_count(),
            other => other
        }
    }

    pub fn edges_count(&self) -> usize {
        let overriden = self.key_frame.edges_count();
        match overriden {
            0 => self.parent().edges_count(),
            other => other
        }
    }

    pub fn faces_count(&self) -> usize {
        let overriden = self.key_frame.faces_count();
        match overriden {
            0 => self.parent().faces_count(),
            other => other
        }
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

    
    pub fn vertices_count(&self) -> usize {
        match self {
            Self::Key(core) | Self::NonInheriting{core, ..} => core.vertices_count(),
            Self::Inheriting(child) => child.vertices_count() 
        }
    }

    pub fn edges_count(&self) -> usize {
        match self {
            Self::Key(core) | Self::NonInheriting{core, ..} => core.edges_count(),
            Self::Inheriting(child) => child.edges_count() 
        }
    }

    pub fn faces_count(&self) -> usize {
        match self {
            Self::Key(core) | Self::NonInheriting{core, ..} => core.faces_count(),
            Self::Inheriting(child) => child.faces_count() 
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
