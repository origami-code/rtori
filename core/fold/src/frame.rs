// TODO: add a u32 (u32::MAX) & f32 (a NaN ?) variant which bytemuck convert to the underlying type

use crate::collections::{Lockstep, LockstepNU, SeededOption};

use super::*;

pub trait FrameVertices<'a> {
    fn count(&self) -> usize {0}
    
    fn coords(&self) -> &'a LockstepNU<'a, f32> {&SeededOption(None)} 
    fn adjacent(&self) -> &'a LockstepNU<'a, VertexIndex> {&SeededOption(None)}
    fn edges(&self) -> &'a LockstepNU<'a, EdgeIndex> {&SeededOption(None)}

    /// `vertices_faces` in the fold specification
    /// 
    /// For each vertex, an array of face IDs for the faces incident to the vertex
    /// Possibly including None (null).
    fn faces(&self) -> &'a LockstepNU<'a, Option<FaceIndex>> {&SeededOption(None)}

    ///  `rtori:vertices_mass` in the rtori extension of the fold specification
    fn rtori_vertices_mass(&self) -> &'a Lockstep<'a, f32> {&SeededOption(None)}
}

pub trait FrameEdges<'a> {
    fn count(&self) -> usize {0}
    
    /// `edges_vertices` in the fold specification
    /// 
    /// For each edge, an array [u, v] of two vertex IDs for the two endpoints of the edge.
    /// This effectively defines the orientation of the edge, from u to v.
    /// (This orientation choice is arbitrary, but is used to define the ordering of edges_faces.)
    /// Recommended in frames having any edges_... property (e.g., to represent mountain-valley assignment).
    fn vertices(&self) -> &'a Lockstep<'a, EdgeVertexIndices> {&SeededOption(None)}
    fn faces(&self) -> &'a LockstepNU<'a, Option<VertexIndex>> {&SeededOption(None)}
    fn assignment(&self) -> &'a Lockstep<'a, EdgeAssignment> {&SeededOption(None)}
    fn fold_angle(&self) -> &'a Lockstep<'a, Option<f32>> {&SeededOption(None)}
    fn length(&self) -> &'a Lockstep<'a, f32> {&SeededOption(None)}

    
    /// `rtori:edges_creaseStiffness` in the rtori extension of the fold specification
    fn crease_stiffness(&self) -> &'a Lockstep<'a, Option<f32>> {&SeededOption(None)}

    /// `rtori:edges_axialStiffness` in the rtori extension of the fold specification
    fn axial_stiffness(&self) -> &'a Lockstep<'a, Option<f32>> {&SeededOption(None)}
}

pub trait FrameFaces<'a> {
    fn count(&self) -> usize {0}
    
    /// the vertices that comprise the face
    fn vertices(&self) -> &'a LockstepNU<'a, VertexIndex> {&SeededOption(None)}

    /// For each face, an array of edge IDs for the edges around the face in counterclockwise order.
    /// In addition to the matching cyclic order, faces_vertices and faces_edges should align in start
    /// so that faces_edges[f][i] is the edge connecting faces_vertices[f][i]
    /// and faces_vertices[f][(i+1)%d] where d is the degree of face f.
    fn edges(&self) -> &'a LockstepNU<'a, EdgeIndex> {&SeededOption(None)}

    /// For each face, an array of face IDs for the faces sharing edges around the face, possibly including nulls.
    /// If the frame is a manifold, the faces should be listed in counterclockwise order and in the same linear order as faces_edges (if it is specified):
    ///     f and faces_faces[f][i] should be the faces incident to the edge faces_edges[f][i],
    ///     unless that edge has no face on the other side, in which case faces_faces[f][i] should be null.
    /// Optimized for no more than 8 faces sharing edges with each face
    fn faces(&self) -> &'a LockstepNU<'a, Option<FaceIndex>> {&SeededOption(None)}

    /// For each face, an array of uv indices corresponding to the vertices of the same index
    /// That is, for `rtori:faces_uvs[n][a] = k` assigns to the vertex index `faces_edges[n][a]` the uv `rtori:uvs[k]`
    fn uvs(&self) -> &'a LockstepNU<'a, u32> {&SeededOption(None)}
}

pub trait Frame<'a> {
    type Vertices: FrameVertices<'a>;
    type Edges: FrameEdges<'a>;
    type Faces: FrameFaces<'a>;

    fn vertices(&self) -> Self::Vertices;
    fn edges(&self) -> Self::Edges;
    fn faces(&self) -> Self::Faces;
}




