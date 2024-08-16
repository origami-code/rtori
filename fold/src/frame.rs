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
    pub unit: Option<Vec<String>>,
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

pub enum Frame<'a> {
    Key(&'a FrameCore),
    NonKey(&'a NonKeyFrame),
}
