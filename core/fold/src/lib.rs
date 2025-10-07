#![cfg_attr(not(test), no_std)]
//#![feature(impl_trait_in_assoc_type)]
//#![feature(coroutines)]
//#![feature(coroutine_trait)]
//#![feature(stmt_expr_attributes)]

extern crate alloc;

mod indices;
pub use indices::*;

pub mod collections;

mod common;
use common::*;

mod edges;
pub use edges::*;

mod layers;
pub use layers::*;

mod frame;
pub use frame::*;

pub mod macros;

mod deser;
pub use deser::Seed;

mod file;
pub use file::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Field {
    FacesVertices,
    EdgesVertices,
    VerticesFaces,
    VerticesEdges,
    VerticesCoords,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_deserialization(contents: &str) {
        let bump = bumpalo::Bump::new();
        let seed = crate::deser::Seed::from_bump(&bump);
        let mut deser = serde_json::de::Deserializer::from_str(contents);
        let output = <file::File as serde_seeded::DeserializeSeeded<_>>::deserialize_seeded(
            &seed, &mut deser,
        );
        println!("Output: {:#?}", output);
    }

    macro_rules! declare_file(
        ($const_name:ident, $test_name:ident, $file:expr) => {
            const $const_name: &'static str = include_str!($file);

            #[test]
            pub fn $test_name() {
                test_deserialization($const_name);
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
    declare_file!(
        THIRTEEN_HORNS,
        deserialize_thirteen_horns,
        "../testdata/13-horns-123-vertices.fold"
    );
    declare_file!(
        THIRTEEN_HORNS_AUGMENTED,
        deserialize_thirteen_horns_augmented,
        "../testdata/13-horns-123-vertices-augmented.fold"
    );
    declare_file!(
        THIRTEEN_HORNS_AUGMENTED_TRIANGULATED,
        deserialize_thirteen_horns_augmented_triangulated,
        "../testdata/13-horns-123-vertices-augmented-triangulated.fold"
    );
}
