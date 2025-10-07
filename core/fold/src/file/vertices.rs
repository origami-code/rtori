use crate::collections::{Lockstep, LockstepNU};

use crate::indices::*;

#[derive(serde_seeded::DeserializeSeeded, Debug, Clone, serde::Serialize)]
#[seeded(de(seed(crate::deser::Seed<'alloc>)))]
pub struct VertexInformation<'alloc> {
    #[serde(rename = "vertices_coords")]
    pub coords: LockstepNU<'alloc, f32>,

    #[serde(rename = "vertices_vertices")]
    pub adjacent: LockstepNU<'alloc, VertexIndex>,

    #[serde(rename = "vertices_edges")]
    pub edges: LockstepNU<'alloc, EdgeIndex>,

    /// see [crate::FrameVertices::faces]
    #[serde(rename = "vertices_faces")]
    pub faces: LockstepNU<'alloc, Option<FaceIndex>>,

    /// see [crate::FrameVertices::sim_weight]
    #[serde(rename = "rtori:vertices_mass")]
    pub rtori_vertices_mass: Lockstep<'alloc, f32>,
}
static_assertions::assert_impl_all!(VertexInformation<'static>: serde_seeded::DeserializeSeeded<'static, crate::deser::Seed<'static>>);

use crate::implement_member;

impl<'a> crate::frame::FrameVertices<'a> for &'a VertexInformation<'a> {
    fn count(&self) -> usize {
        self.coords.as_ref().map(|c| c.len()).unwrap_or(0)
    }

    implement_member!(coords, &'a LockstepNU<'a, f32>);
    implement_member!(adjacent, &'a LockstepNU<'a, VertexIndex>);
    implement_member!(edges, &'a LockstepNU<'a, EdgeIndex>);
    implement_member!(faces, &'a LockstepNU<'a, Option<FaceIndex>>);
    implement_member!(rtori_vertices_mass, &'a Lockstep<'a, f32>);
}
