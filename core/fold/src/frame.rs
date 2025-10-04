use alloc::borrow::Cow;

use crate::Lockstep;

use super::*;

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct FrameMetadata {
    #[serde(rename = "frame_title")]
    pub title: Option<String>,

    #[serde(rename = "frame_description")]
    pub description: Option<String>,

    #[serde(rename = "frame_classes")]
    pub classes: Option<Vec<String>>,

    #[serde(rename = "frame_attributes")]
    pub attributes: Option<Vec<String>>,

    #[serde(rename = "frame_unit")]
    pub unit: Option<String>,
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct FrameCore {
    #[serde(flatten)]
    pub metadata: FrameMetadata,

    #[serde(flatten)]
    pub vertices: VertexInformation,

    #[serde(flatten)]
    pub edges: EdgeInformation,

    #[serde(flatten)]
    pub faces: FaceInformation,

    #[serde(flatten)]
    pub layering: LayerInformation,

    pub uvs: Lockstep<[f32; 2]>,
}

impl FrameCore {
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

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct NonKeyFrame {
    #[serde(flatten)]
    pub frame: FrameCore,
    #[serde(rename = "frame_parent")]
    pub parent: Option<FrameIndex>,
    #[serde(rename = "frame_inherit")]
    pub inherit: Option<bool>,
}

pub struct InheritingFrame<'a> {
    frames: &'a [NonKeyFrame],
    key_frame: &'a FrameCore,
    frame_index: u16
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

pub enum FrameRef<'a> {
    Key(&'a FrameCore),
    NonInheriting{core: &'a FrameCore, parent: Option<u16>},
    Inheriting(InheritingFrame<'a>)
}

impl<'a> FrameRef<'a> {
    pub fn create(frames: &'a [NonKeyFrame], key_frame: &'a FrameCore, frame_index: u16) -> Option<Self> {
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
