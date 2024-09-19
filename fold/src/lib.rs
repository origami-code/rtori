#![feature(impl_trait_in_assoc_type)]
#![feature(coroutines)]
#![feature(coroutine_trait)]
#![feature(stmt_expr_attributes)]

mod indices;
pub use indices::*;

mod handful;
use handful::Handful;

mod lockstep;
use lockstep::Lockstep;

mod common;

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

pub mod macros;

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct FileMetadata {
    #[serde(rename = "file_spec")]
    pub spec: Option<u32>,
    #[serde(rename = "file_creator")]
    pub creator: Option<String>,
    #[serde(rename = "file_author")]
    pub author: Option<String>,
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct File {
    #[serde(flatten)]
    pub file_metadata: Option<FileMetadata>,

    #[serde(rename = "file_frames")]
    pub frames: Option<Vec<NonKeyFrame>>,

    #[serde(flatten)]
    pub key_frame: FrameCore,
}

impl File {
    pub fn frame<'a>(&'a self, index: FrameIndex) -> Option<Frame<'a>> {
        match index {
            0 => Some(Frame::Key(&self.key_frame)),
            other => self
                .frames
                .as_ref()
                .and_then(|frame_vec| frame_vec.get(usize::from(other - 1)))
                .map(|frame| Frame::NonKey(frame)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    const SIMPLE: &'static str = include_str!("../testdata/simple.fold");
    #[test]

    pub fn deserialize_simple() {
        let output = serde_json::from_str::<File>(SIMPLE).unwrap();
        println!("Output: {:#?}", output);
    }
}
