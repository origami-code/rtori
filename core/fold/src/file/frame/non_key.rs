use super::FrameCore;
use crate::FrameIndex;

#[derive(serde_seeded::DeserializeSeeded, Debug, Clone, serde::Serialize)]
#[seeded(de(seed(crate::deser::Seed<Alloc>), override_bounds(Alloc: Clone)))]
pub struct NonKeyFrame<Alloc: core::alloc::Allocator> {
    #[serde(flatten)]
    pub frame: FrameCore<Alloc>,
    #[serde(rename = "frame_parent")]
    pub parent: Option<FrameIndex>,
    #[serde(rename = "frame_inherit")]
    pub inherit: Option<bool>,
}
