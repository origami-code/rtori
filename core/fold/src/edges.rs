use super::indices::*;

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, serde::Serialize, serde::Deserialize,
)]
#[cfg_attr(
    feature = "bytemuck",
    derive(bytemuck::AnyBitPattern, bytemuck::NoUninit)
)]
#[repr(transparent)]
pub struct EdgeVertexIndices(pub [VertexIndex; 2]);

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, serde::Serialize, serde::Deserialize,
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
