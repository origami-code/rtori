use crate::collections::{SeededOption, Lockstep, String};

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
