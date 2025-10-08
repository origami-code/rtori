use crate::collections::VecNU;
use crate::indices::*;

#[derive(serde_seeded::DeserializeSeeded, Debug, Clone, serde::Serialize, Default)]
#[seeded(de(seed(crate::deser::Seed<Alloc>), override_bounds(Alloc: Clone)))]
pub struct FaceInformation<Alloc: core::alloc::Allocator> {
    /// see [crate::FrameFaces::vertices]
    #[serde(rename = "faces_vertices")]
    pub vertices: VecNU<VertexIndex, Alloc>,

    /// see [crate::FrameFaces::edges]
    #[serde(rename = "faces_edges")]
    pub edges: VecNU<EdgeIndex, Alloc>,

    /// see [crate::FrameFaces::faces]
    #[serde(rename = "faces_faces")]
    pub faces: VecNU<MaskableFaceIndex, Alloc>,

    /// see [crate::FrameFaces::uvs]
    #[serde(rename = "rtori:faces_uvs")]
    pub uvs: VecNU<u32, Alloc>,
}
crate::assert_deserializable!(assert_faces, FaceInformation<Alloc>);

use crate::collections::NUSlice;
use crate::implement_member;

impl<'a, Alloc> crate::frame::FrameFaces<'a> for &'a FaceInformation<Alloc>
where
    Alloc: core::alloc::Allocator + 'a,
{
    fn count(&self) -> usize {
        self.vertices.len()
    }

    implement_member!(vertices, NUSlice<'a, VertexIndex>);
    implement_member!(edges, NUSlice<'a, EdgeIndex>);
    implement_member!(faces, NUSlice<'a, MaskableFaceIndex>);
    implement_member!(uvs, NUSlice<'a, u32>);
}
