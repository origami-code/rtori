use crate::collections::{Lockstep, LockstepNU};

use crate::edges::{EdgeAssignment, EdgeVertexIndices};
use crate::indices::*;

#[derive(serde_seeded::DeserializeSeeded, Debug, Clone, Default, serde::Serialize)]
#[seeded(de(seed(crate::deser::Seed<'alloc>)))]
pub struct EdgeInformation<'alloc> {
    /// see [crate::FrameEdges::vertices]
    #[serde(rename = "edges_vertices")]
    pub vertices: Lockstep<'alloc, EdgeVertexIndices>,

    /// see [crate::FrameEdges::faces]
    #[serde(rename = "edges_faces")]
    pub faces: LockstepNU<'alloc, Option<VertexIndex>>, // an edge will generally have no more than two faces, though it can happen

    /// see [crate::FrameEdges::assignment]
    #[serde(rename = "edges_assignment")]
    pub assignment: Lockstep<'alloc, EdgeAssignment>,

    /// see [crate::FrameEdges::fold_angle]
    #[serde(rename = "edges_foldAngle")]
    pub fold_angle: Lockstep<'alloc, Option<f32>>,

    /// see [crate::FrameEdges::length]
    #[serde(rename = "edges_length")]
    pub length: Lockstep<'alloc, f32>,

    /// see [crate::FrameEdges::crease_stiffness]
    #[serde(rename = "rtori:edges_creaseStiffness")]
    pub crease_stiffness: Lockstep<'alloc, Option<f32>>,

    /// see [crate::FrameEdges::axial_stiffness]
    #[serde(rename = "rtori:edges_axialStiffness")]
    pub axial_stiffness: Lockstep<'alloc, Option<f32>>,
}

use crate::implement_member;

impl<'a> crate::frame::FrameEdges<'a> for &'a EdgeInformation<'a> {
    fn count(&self) -> usize {
        self.vertices.as_ref().map(|c| c.len()).unwrap_or(0)
    }

    implement_member!(vertices, &'a Lockstep<'a, EdgeVertexIndices>);
    implement_member!(faces, &'a LockstepNU<'a, Option<VertexIndex>>);
    implement_member!(assignment, &'a Lockstep<'a, EdgeAssignment>);
    implement_member!(fold_angle, &'a Lockstep<'a, Option<f32>>);
    implement_member!(length, &'a Lockstep<'a, f32>);
    implement_member!(axial_stiffness, &'a Lockstep<'a, Option<f32>>);
}
