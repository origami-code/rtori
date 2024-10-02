use super::common::*;
use super::indices::*;
use super::Handful;
use super::Lockstep;

#[derive(Debug, Clone, PartialEq, PartialOrd, serde::Deserialize, serde::Serialize)]
#[repr(transparent)]
pub struct Vertex(pub Handful<f32, 3>);

impl core::ops::Deref for Vertex {
    type Target = [f32];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct VertexInformation {
    #[serde(rename = "vertices_coords")]
    pub coords: Lockstep<Vertex>,

    #[serde(rename = "vertices_vertices")]
    pub adjacent: Lockstep<Handful<VertexIndex, 8>>,

    #[serde(rename = "vertices_edges")]
    pub edges: Lockstep<Handful<EdgeIndex, 8>>,

    /// For each vertex, an array of face IDs for the faces incident to the vertex
    /// Possibly including None (null).
    #[serde(rename = "vertices_faces")]
    pub faces: Lockstep<Handful<Option<FaceIndex>, 8>>,

    #[serde(rename = "rtori:vertices_mass")]
    pub sim_weight: Lockstep<f32>,
}

impl VertexInformation {
    pub fn count(&self) -> usize {
        self.coords.as_ref().map(|c| c.len()).unwrap_or(0)
    }
}
