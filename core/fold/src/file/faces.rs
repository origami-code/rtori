use crate::indices::*;
use crate::collections::LockstepNU;

#[derive(serde_seeded::DeserializeSeeded, Debug, Clone, serde::Serialize)]
#[seeded(de(seed(crate::deser::Seed<'alloc>)))]
pub struct FaceInformation<'alloc> {
    /// see [crate::FrameFaces::vertices]
    #[serde(rename = "faces_vertices")]
    pub vertices: LockstepNU<'alloc, VertexIndex>,

    /// see [crate::FrameFaces::edges]
    #[serde(rename = "faces_edges")]
    pub edges: LockstepNU<'alloc, EdgeIndex>,

    /// see [crate::FrameFaces::faces]
    #[serde(rename = "faces_faces")]
    pub faces: LockstepNU<'alloc, Option<FaceIndex>>,

    /// see [crate::FrameFaces::uvs]
    #[serde(rename = "rtori:faces_uvs")]
    pub uvs: LockstepNU<'alloc, u32>,
}

use crate::implement_member;

impl<'a> crate::frame::FrameFaces<'a> for &'a FaceInformation<'a> {
    fn count(&self) -> usize {
        self.vertices.as_ref().map(|c| c.len()).unwrap_or(0)
    }
    
    implement_member!(vertices, &'a LockstepNU<'a, VertexIndex>);
    implement_member!(edges, &'a LockstepNU<'a, EdgeIndex>);
    implement_member!(faces, &'a LockstepNU<'a, Option<FaceIndex>>);
    implement_member!(uvs, &'a LockstepNU<'a, u32>);
}