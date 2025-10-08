use super::*;

use crate::Frame;

mod metadata;
pub use metadata::FrameMetadata;

mod non_key;
pub use non_key::NonKeyFrame;

mod inheriting;
pub use inheriting::InheritingFrame;

mod r#ref;
pub use r#ref::FrameRef;

#[derive(serde_seeded::DeserializeSeeded, Debug, Clone, serde::Serialize, Default)]
#[seeded(de(seed(crate::deser::Seed<Alloc>), override_bounds(Alloc: Clone)))]
pub struct FrameCore<Alloc: core::alloc::Allocator> {
    #[serde(flatten)]
    pub metadata: FrameMetadata<Alloc>,

    #[serde(flatten)]
    pub vertices: VertexInformation<Alloc>,

    #[serde(flatten)]
    pub edges: EdgeInformation<Alloc>,

    #[serde(flatten)]
    pub faces: FaceInformation<Alloc>,

    #[serde(flatten)]
    pub layering: LayerInformation<Alloc>,

    pub uvs: crate::collections::VecU<[f32; 2], Alloc>,
}

impl<'a, Alloc> Frame<'a> for &'a FrameCore<Alloc>
where
    Alloc: core::alloc::Allocator,
{
    type Vertices = &'a VertexInformation<Alloc>;
    type Edges = &'a EdgeInformation<Alloc>;
    type Faces = &'a FaceInformation<Alloc>;

    fn vertices(&self) -> Self::Vertices {
        &self.vertices
    }

    fn edges(&self) -> Self::Edges {
        &self.edges
    }

    fn faces(&self) -> Self::Faces {
        &self.faces
    }
}
