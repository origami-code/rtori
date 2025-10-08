use crate::collections;

#[derive(serde_seeded::DeserializeSeeded, Debug, Clone, serde::Serialize)]
#[seeded(de(seed(crate::deser::Seed<Alloc>), bounds(Alloc: Clone)))]
pub struct FileMetadata<Alloc: core::alloc::Allocator> {
    #[serde(rename = "file_spec")]
    pub spec: Option<u32>,

    #[serde(rename = "file_creator")]
    pub creator: collections::SeededOption<collections::String<Alloc>>,

    #[serde(rename = "file_author")]
    pub author: collections::SeededOption<collections::String<Alloc>>,
}
crate::assert_deserializable!(assert_file_metadata, FileMetadata<Alloc>);
