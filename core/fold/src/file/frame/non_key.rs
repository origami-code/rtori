use super::FrameCore;
use crate::FrameIndex;

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
