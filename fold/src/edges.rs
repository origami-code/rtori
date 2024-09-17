use crate::common;

use super::indices::*;

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, serde::Deserialize, serde::Serialize,
)]
#[cfg_attr(
    feature = "bytemuck",
    derive(bytemuck::AnyBitPattern, bytemuck::NoUninit)
)]
#[repr(transparent)]
pub struct EdgeVertexIndices(pub [VertexIndex; 2]);

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, serde::Deserialize, serde::Serialize,
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

#[derive(Debug, Clone, Default, serde::Deserialize, serde::Serialize)]
pub struct EdgeInformation {
    /// For each edge, an array [u, v] of two vertex IDs for the two endpoints of the edge.
    /// This effectively defines the orientation of the edge, from u to v.
    /// (This orientation choice is arbitrary, but is used to define the ordering of edges_faces.)
    /// Recommended in frames having any edges_... property (e.g., to represent mountain-valley assignment).
    #[serde(rename = "edges_vertices")]
    pub vertices: Option<Vec<EdgeVertexIndices>>,

    #[serde(rename = "edges_faces")]
    pub faces: Option<Vec<common::SmallVec<[VertexIndex; 3]>>>,

    #[serde(rename = "edges_assignment")]
    pub assignments: Option<Vec<EdgeAssignment>>,

    #[serde(rename = "edges_foldAngle")]
    pub fold_angles: Option<Vec<f32>>,

    #[serde(rename = "edges_length")]
    pub length: Option<Vec<f32>>,
}
