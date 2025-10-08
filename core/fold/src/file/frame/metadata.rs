use crate::collections::{SeededOption, String, VecU};

#[derive(serde_seeded::DeserializeSeeded, Debug, Clone, serde::Serialize)]
#[seeded(de(seed(crate::deser::Seed<Alloc>), bounds(Alloc: Clone)))]
pub struct FrameMetadata<Alloc: core::alloc::Allocator> {
    #[serde(rename = "frame_title")]
    pub title: SeededOption<String<Alloc>>,

    #[serde(rename = "frame_description")]
    pub description: SeededOption<String<Alloc>>,

    #[serde(rename = "frame_classes")]
    pub classes: VecU<String<Alloc>, Alloc>,

    #[serde(rename = "frame_attributes")]
    pub attributes: VecU<String<Alloc>, Alloc>,

    #[serde(rename = "frame_unit")]
    pub unit: SeededOption<String<Alloc>>,
}
