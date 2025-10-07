use super::*;

use crate::collections::Lockstep;
use crate::Frame;

mod metadata;
pub use metadata::FrameMetadata;

mod non_key;
pub use non_key::NonKeyFrame;

mod inheriting;
pub use inheriting::InheritingFrame;

mod r#ref;
pub use r#ref::FrameRef;

#[derive(serde_seeded::DeserializeSeeded, Debug, Clone, serde::Serialize)]
#[seeded(de(seed(crate::deser::Seed<'alloc>)))]
pub struct FrameCore<'alloc> {
    #[serde(flatten)]
    pub metadata: FrameMetadata<'alloc>,

    #[serde(flatten)]
    pub vertices: VertexInformation<'alloc>,

    #[serde(flatten)]
    pub edges: EdgeInformation<'alloc>,

    #[serde(flatten)]
    pub faces: FaceInformation<'alloc>,

    #[serde(flatten)]
    pub layering: LayerInformation<'alloc>,

    pub uvs: Lockstep<'alloc, [f32; 2]>,
}

impl<'a> Frame<'a> for &'a FrameCore<'a> {
    type Vertices = &'a VertexInformation<'a>;
    type Edges = &'a EdgeInformation<'a>;
    type Faces = &'a FaceInformation<'a>;

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
