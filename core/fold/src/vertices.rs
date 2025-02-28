use crate::collections::{Handful, Lockstep, LockstepNU};

use super::indices::*;


#[derive(serde_seeded::DeserializeSeeded, Debug, Clone, serde::Serialize)]
#[seeded(de(seed(crate::deser::Seed<'alloc>)))]
pub struct VertexInformation<'alloc> {
    #[serde(rename = "vertices_coords")]
    pub coords: LockstepNU<'alloc, f32>,

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
static_assertions::assert_impl_all!(VertexInformation<'static>: serde_seeded::DeserializeSeeded<'static, crate::deser::Seed<'static>>);

impl VertexInformation<'_> {
    pub fn count(&self) -> usize {
        self.coords.as_ref().map(|c| c.len()).unwrap_or(0)
    }
}
