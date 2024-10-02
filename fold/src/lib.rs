#![cfg_attr(not(test), no_std)]
#![feature(impl_trait_in_assoc_type)]
#![feature(coroutines)]
#![feature(coroutine_trait)]
#![feature(stmt_expr_attributes)]

extern crate alloc;

mod indices;
pub use indices::*;

mod handful;
use handful::Handful;

mod lockstep;
use lockstep::Lockstep;

mod common;
use common::*;

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

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Field {
    FacesVertices,
    EdgesVertices,
    VerticesFaces,
    VerticesEdges,
    VerticesCoords,
}

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

    macro_rules! declare_file(
        ($const_name:ident, $test_name:ident, $file:expr) => {
            const $const_name: &'static str = include_str!($file);

            #[test]
            pub fn $test_name() {
                let output = serde_json::from_str::<File>($const_name).unwrap();
                println!("Output: {:#?}", output);
            }
        }
    );

    declare_file!(SIMPLE, deserialize_simple, "../testdata/simple.fold");
    declare_file!(BOX, deserialize_box, "../testdata/box.fold");
    declare_file!(
        DIAGONAL_CP,
        deserialize_diagonal_cp,
        "../testdata/diagonal-cp.fold"
    );
    declare_file!(
        DIAGONAL_FOLDED,
        deserialize_diagonal_folded,
        "../testdata/diagonal-folded.fold"
    );
    declare_file!(
        ONE_VERTEX,
        deserialize_one_vertex,
        "../testdata/one_vertex.fold"
    );
    declare_file!(
        SQUARE_TWIST,
        deserialize_square_twist,
        "../testdata/squaretwist.fold"
    );
}
