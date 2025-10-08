use crate::collections::{VecNU, VecU};

use crate::indices::*;

#[derive(serde_seeded::DeserializeSeeded, Debug, Clone, serde::Serialize)]
#[seeded(de(seed(crate::deser::Seed<Alloc>), bounds(Alloc: Clone)))]
pub struct VertexInformation<Alloc: core::alloc::Allocator> {
    #[serde(rename = "vertices_coords")]
    pub coords: VecNU<f32, Alloc>,

    #[serde(rename = "vertices_vertices")]
    pub adjacent: VecNU<VertexIndex, Alloc>,

    #[serde(rename = "vertices_edges")]
    pub edges: VecNU<EdgeIndex, Alloc>,

    /// see [crate::FrameVertices::faces]
    #[serde(rename = "vertices_faces")]
    pub faces: VecNU<MaskableFaceIndex, Alloc>,

    /// see [crate::FrameVertices::sim_weight]
    #[serde(rename = "rtori:vertices_mass")]
    pub rtori_vertices_mass: VecU<f32, Alloc>,
}
crate::assert_deserializable!(assert_vertices, VertexInformation<Alloc>);

use crate::collections::NUSlice;
use crate::implement_member;

impl<'a, Alloc> crate::frame::FrameVertices<'a> for &'a VertexInformation<Alloc>
where
    Alloc: core::alloc::Allocator + 'a,
{
    fn count(&self) -> usize {
        self.coords.len()
    }

    implement_member!(coords, NUSlice<'a, f32>);
    implement_member!(adjacent, NUSlice<'a, VertexIndex>);
    implement_member!(edges, NUSlice<'a, EdgeIndex>);
    implement_member!(faces, NUSlice<'a, MaskableFaceIndex>);
    implement_member!(rtori_vertices_mass, &'a [f32]);
}
