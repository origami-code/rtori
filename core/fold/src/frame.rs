// TODO: add a u32 (u32::MAX) & f32 (a NaN ?) variant which bytemuck convert to the underlying type

use alloc::borrow::Cow;

use crate::collections::{Lockstep, SeededOption, String};

use super::*;


pub trait FrameVertices<'a> {
    fn count(&self) -> usize {0}
    
    fn coords(&self) -> &'a crate::collections::LockstepNU<'a, f32> {&SeededOption(None)} 
    fn adjacent(&self) -> &'a crate::collections::LockstepNU<'a, VertexIndex> {&SeededOption(None)}
    fn edges(&self) -> &'a crate::collections::LockstepNU<'a, EdgeIndex> {&SeededOption(None)}
    fn faces(&self) -> &'a crate::collections::LockstepNU<'a, Option<FaceIndex>> {&SeededOption(None)}
}

pub trait FrameEdges<'a> {
    fn count(&self) -> usize {0}
    
    fn vertices(&self) -> &'a crate::collections::Lockstep<'a, EdgeVertexIndices> {&SeededOption(None)}
    fn faces(&self) -> &'a crate::collections::LockstepNU<'a, Option<VertexIndex>> {&SeededOption(None)}
    fn assignment(&self) -> &'a crate::collections::Lockstep<'a, EdgeAssignment> {&SeededOption(None)}
    fn fold_angle(&self) -> &'a crate::collections::Lockstep<'a, Option<f32>> {&SeededOption(None)}
    fn length(&self) -> &'a crate::collections::Lockstep<'a, f32> {&SeededOption(None)}
}

pub trait FrameFaces<'a> {
    fn count(&self) -> usize {0}
    
    fn vertices(&self) -> &'a crate::collections::LockstepNU<'a, VertexIndex> {&SeededOption(None)}
    fn edges(&self) -> &'a crate::collections::LockstepNU<'a, EdgeIndex> {&SeededOption(None)}
    fn uvs(&self) -> &'a crate::collections::LockstepNU<'a, u32> {&SeededOption(None)}
}

pub trait Frame<'a> {
    type Vertices: FrameVertices<'a>;
    type Edges: FrameEdges<'a>;
    type Faces: FrameFaces<'a>;

    fn vertices(&self) -> Self::Vertices;
    fn edges(&self) -> Self::Edges;
    fn faces(&self) -> Self::Faces;
}




