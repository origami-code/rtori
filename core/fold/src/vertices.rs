use crate::handful::LockstepNU;

use super::common::*;
use super::indices::*;
use super::Handful;
use super::Lockstep;

#[derive(Debug, Clone, PartialEq, PartialOrd, serde::Serialize)]
#[repr(transparent)]
pub struct Vertex<'alloc>(pub Handful<'alloc, f32, 3>);

impl core::ops::Deref for Vertex<'_> {
    type Target = [f32];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct VertexInformation<'alloc> {
    #[serde(rename = "vertices_coords")]
    pub coords: Lockstep<'alloc, Vertex<'alloc>>,

    #[serde(rename = "vertices_vertices")]
    pub adjacent: LockstepNU<'alloc, VertexIndex>,

    #[serde(rename = "vertices_edges")]
    pub edges: LockstepNU<'alloc, EdgeIndex>,

    /// For each vertex, an array of face IDs for the faces incident to the vertex
    /// Possibly including None (null).
    #[serde(rename = "vertices_faces")]
    pub faces: LockstepNU<'alloc, Option<FaceIndex>>,

    #[serde(rename = "rtori:vertices_mass")]
    pub sim_weight: Lockstep<'alloc, f32>,
}

impl VertexInformation<'_> {
    pub fn count(&self) -> usize {
        self.coords.as_ref().map(|c| c.len()).unwrap_or(0)
    }
}
