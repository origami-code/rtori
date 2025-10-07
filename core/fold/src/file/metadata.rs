use crate::collections;

#[derive(serde_seeded::DeserializeSeeded, Debug, Clone, serde::Serialize)]
#[seeded(de(seed(crate::deser::Seed<'alloc>)))]
pub struct FileMetadata<'alloc> {
    #[serde(rename = "file_spec")]
    pub spec: Option<u32>,

    #[serde(rename = "file_creator")]
    pub creator: collections::SeededOption<collections::String<'alloc>>,
    
    #[serde(rename = "file_author")]
    pub author: collections::SeededOption<collections::String<'alloc>>,
}
static_assertions::assert_impl_all!(FileMetadata<'static>: serde_seeded::DeserializeSeeded<'static, crate::deser::Seed<'static>>);
