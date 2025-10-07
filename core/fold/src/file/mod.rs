mod vertices;
pub use vertices::*;

mod edges;
pub use edges::*;

mod faces;
pub use faces::*;

mod layers;
pub use layers::*;

mod frame;
pub use frame::*;

mod metadata;
pub use metadata::*;

use crate::{collections, FrameIndex};

#[derive(serde_seeded::DeserializeSeeded, Debug, Clone, serde::Serialize)]
#[seeded(de(seed(crate::deser::Seed<'alloc>)))]
pub struct File<'alloc> {
    #[serde(flatten)]
    pub file_metadata: FileMetadata<'alloc>,

    #[serde(rename = "file_frames")]
    pub frames: collections::VecU<'alloc, NonKeyFrame<'alloc>>,

    #[serde(flatten)]
    pub key_frame: FrameCore<'alloc>,
}

static_assertions::assert_impl_all!(FileMetadata<'static>: serde_seeded::DeserializeSeeded<'static, crate::deser::Seed<'static>>);

impl File<'_> {
    pub fn frame<'a>(&'a self, index: FrameIndex) -> Option<FrameRef<'a>> {
        FrameRef::create(&self.frames, &self.key_frame, index)
    }

    pub fn frame_count(&self) -> FrameIndex {
        let nonkey_frame_count = self.frames.len();
        1u16 + u16::try_from(nonkey_frame_count).unwrap()
    }
}

#[macro_export]
macro_rules! implement_member {
    ($member:ident, $type:ty) => {
        fn $member(&self) -> $type {
            &self.$member
        }
    };
}
