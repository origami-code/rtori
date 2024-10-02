use super::common::*;
use super::indices::*;
use crate::Handful;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, serde::Deserialize, serde::Serialize)]
#[repr(transparent)]
pub struct Face(pub Handful<VertexIndex, 4>);

impl core::ops::Deref for Face {
    type Target = [VertexIndex];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct FaceInformation {
    #[serde(rename = "faces_vertices")]
    pub vertices: Option<Vec<Face>>,

    /// For each face, an array of edge IDs for the edges around the face in counterclockwise order.
    /// In addition to the matching cyclic order, faces_vertices and faces_edges should align in start
    /// so that faces_edges[f][i] is the edge connecting faces_vertices[f][i]
    /// and faces_vertices[f][(i+1)%d] where d is the degree of face f.
    #[serde(rename = "faces_edges")]
    pub edges: Option<Vec<Handful<EdgeIndex, 8>>>,

    /// For each face, an array of face IDs for the faces sharing edges around the face, possibly including nulls.
    /// If the frame is a manifold, the faces should be listed in counterclockwise order and in the same linear order as faces_edges (if it is specified):
    ///     f and faces_faces[f][i] should be the faces incident to the edge faces_edges[f][i],
    ///     unless that edge has no face on the other side, in which case faces_faces[f][i] should be null.
    /// Optimized for no more than 8 faces sharing edges with each face
    #[serde(rename = "faces_faces")]
    pub faces: Option<Vec<Handful<Option<FaceIndex>, 8>>>,
}
