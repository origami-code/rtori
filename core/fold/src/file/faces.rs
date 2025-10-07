use crate::indices::*;
use crate::collections::LockstepNU;

#[derive(serde_seeded::DeserializeSeeded, Debug, Clone, serde::Serialize)]
#[seeded(de(seed(crate::deser::Seed<'alloc>)))]
pub struct FaceInformation<'alloc> {
    #[serde(rename = "faces_vertices")]
    pub vertices: LockstepNU<'alloc, VertexIndex>,

    /// For each face, an array of edge IDs for the edges around the face in counterclockwise order.
    /// In addition to the matching cyclic order, faces_vertices and faces_edges should align in start
    /// so that faces_edges[f][i] is the edge connecting faces_vertices[f][i]
    /// and faces_vertices[f][(i+1)%d] where d is the degree of face f.
    #[serde(rename = "faces_edges")]
    pub edges: LockstepNU<'alloc, EdgeIndex>,

    /// For each face, an array of face IDs for the faces sharing edges around the face, possibly including nulls.
    /// If the frame is a manifold, the faces should be listed in counterclockwise order and in the same linear order as faces_edges (if it is specified):
    ///     f and faces_faces[f][i] should be the faces incident to the edge faces_edges[f][i],
    ///     unless that edge has no face on the other side, in which case faces_faces[f][i] should be null.
    /// Optimized for no more than 8 faces sharing edges with each face
    #[serde(rename = "faces_faces")]
    pub faces: LockstepNU<'alloc, Option<FaceIndex>>,

    /// For each face, an array of uv indices corresponding to the vertices of the same index
    /// That is, for `rtori:faces_uvs[n][a] = k` assigns to the vertex index `faces_edges[n][a]` the uv `rtori:uvs[k]`
    #[serde(rename = "rtori:faces_uvs")]
    pub uvs: LockstepNU<'alloc, u32>,
}

impl<'a> crate::frame::FrameFaces<'a> for &'a FaceInformation<'a> {
    fn count(&self) -> usize {
        self.vertices.as_ref().map(|c| c.len()).unwrap_or(0)
    }
    
    fn vertices(&self) -> &'a crate::collections::LockstepNU<'a, VertexIndex> {
        &self.vertices
    }

    fn edges(&self) -> &'a crate::collections::LockstepNU<'a, EdgeIndex> {
        &self.edges
    }

    fn uvs(&self) -> &'a crate::collections::LockstepNU<'a, u32> {
        &self.uvs
    }
}