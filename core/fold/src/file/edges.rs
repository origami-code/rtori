use crate::collections::{MaskableFloat, VecNU, VecU};

use crate::edges::{EdgeAssignment, EdgeVertexIndices};
use crate::indices::*;

#[derive(serde_seeded::DeserializeSeeded, Debug, Clone, serde::Serialize)]
#[seeded(de(seed(crate::deser::Seed<Alloc>), bounds(Alloc: Clone)))]
pub struct EdgeInformation<Alloc: core::alloc::Allocator> {
    /// see [crate::FrameEdges::vertices]
    #[serde(rename = "edges_vertices")]
    pub vertices: VecU<EdgeVertexIndices, Alloc>,

    /// see [crate::FrameEdges::faces]
    #[serde(rename = "edges_faces")]
    pub faces: VecNU<MaskableVertexIndex, Alloc>, // an edge will generally have no more than two faces, though it can happen

    /// see [crate::FrameEdges::assignment]
    #[serde(rename = "edges_assignment")]
    pub assignment: VecU<EdgeAssignment, Alloc>,

    /// see [crate::FrameEdges::fold_angle]
    #[serde(rename = "edges_foldAngle")]
    pub fold_angle: VecU<MaskableFloat, Alloc>,

    /// see [crate::FrameEdges::length]
    #[serde(rename = "edges_length")]
    pub length: VecU<f32, Alloc>,

    /// see [crate::FrameEdges::crease_stiffness]
    #[serde(rename = "rtori:edges_creaseStiffness")]
    pub crease_stiffness: VecU<MaskableFloat, Alloc>,

    /// see [crate::FrameEdges::axial_stiffness]
    #[serde(rename = "rtori:edges_axialStiffness")]
    pub axial_stiffness: VecU<MaskableFloat, Alloc>,
}
crate::assert_deserializable!(assert_edges, EdgeInformation<Alloc>);

use crate::collections::NUSlice;
use crate::implement_member;

impl<'a, Alloc> crate::frame::FrameEdges<'a> for &'a EdgeInformation<Alloc>
where
    Alloc: core::alloc::Allocator + 'a,
{
    fn count(&self) -> usize {
        self.vertices.len()
    }

    implement_member!(vertices, &'a [EdgeVertexIndices]);
    implement_member!(faces, NUSlice<'a, MaskableVertexIndex>);
    implement_member!(assignment, &'a [EdgeAssignment]);
    implement_member!(fold_angle, &'a [MaskableFloat]);
    implement_member!(length, &'a [f32]);
    implement_member!(axial_stiffness, &'a [MaskableFloat]);
}
