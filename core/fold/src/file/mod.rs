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

#[derive(serde_seeded::DeserializeSeeded, Debug, Clone, serde::Serialize, Default)]
#[seeded(de(seed(crate::deser::Seed<Alloc>), override_bounds(Alloc: Clone)))]
pub struct File<Alloc: core::alloc::Allocator> {
    #[serde(flatten)]
    pub file_metadata: FileMetadata<Alloc>,

    #[serde(rename = "file_frames")]
    pub frames: collections::VecU<NonKeyFrame<Alloc>, Alloc>,

    #[serde(flatten)]
    pub key_frame: FrameCore<Alloc>,
}

crate::assert_deserializable!(assert_file, File<Alloc>);

impl<Alloc> File<Alloc>
where
    Alloc: core::alloc::Allocator,
{
    pub fn frame<'a>(&'a self, index: FrameIndex) -> Option<FrameRef<'a, Alloc>> {
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
            crate::collections::AsSlice::as_slice(&self.$member)
        }
    };
}
