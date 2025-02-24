use crate::handful::LockstepNU;
use crate::Handful;
use crate::Lockstep;

use super::indices::*;

#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    serde::Serialize,
    serde::Deserialize
)]
#[cfg_attr(
    feature = "bytemuck",
    derive(bytemuck::AnyBitPattern, bytemuck::NoUninit)
)]
#[repr(transparent)]
pub struct EdgeVertexIndices(pub [VertexIndex; 2]);

#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    serde::Serialize,
    serde::Deserialize
)]
pub enum EdgeAssignment {
    /// Border/boundary edge
    B,

    /// Montain Crease
    M,

    /// Valley Crease
    V,

    /// Unassigned/Unknown crease
    U,

    /// Cut/slit edge
    C,

    /// Join edge
    J,

    /// Facet
    F,
}

#[derive(Debug, Clone, Default, serde::Serialize)]
pub struct EdgeInformation<'alloc> {
    /// For each edge, an array [u, v] of two vertex IDs for the two endpoints of the edge.
    /// This effectively defines the orientation of the edge, from u to v.
    /// (This orientation choice is arbitrary, but is used to define the ordering of edges_faces.)
    /// Recommended in frames having any edges_... property (e.g., to represent mountain-valley assignment).
    #[serde(rename = "edges_vertices")]
    pub vertices: Lockstep<'alloc, EdgeVertexIndices>,

    #[serde(rename = "edges_faces")]
    // TODO: SmallVec optimization for this one
    pub faces: LockstepNU<'alloc, Option<VertexIndex>>, // an edge will generally have no more than two faces, though it can happen

    #[serde(rename = "edges_assignment")]
    pub assignments: Lockstep<'alloc, EdgeAssignment>,

    #[serde(rename = "edges_foldAngle")]
    pub fold_angles: Lockstep<'alloc, Option<f32>>,

    #[serde(rename = "edges_length")]
    pub length: Lockstep<'alloc, f32>,

    #[serde(rename = "rtori:edges_creaseStiffness")]
    pub crease_stiffness: Lockstep<'alloc, Option<f32>>,

    #[serde(rename = "rtori:edges_axialStiffness")]
    pub axial_stiffness: Lockstep<'alloc, Option<f32>>,
}

impl EdgeInformation<'_> {
    pub fn count(&self) -> usize {
        self.vertices.as_ref().map(|c| c.len()).unwrap_or(0)
    }
}
