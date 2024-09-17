use fold::EdgeAssignment;

pub mod creases;
pub mod triangulation;

pub type Vector3F = [f32; 3];
pub type Vector3U = [u32; 3];
pub type Vector2U = [u32; 2];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FoldAssignment {
    Valley,
    Mountain,
    Facet,
    Boundary,
    Other,
}

pub trait Proxy {
    type Output;
    fn count(&self) -> usize;
    fn get(&self, idx: usize) -> Option<Self::Output>;
    fn iter(&self) -> impl Iterator<Item = Self::Output>;
}

impl<T> Proxy for &[T]
where
    T: Copy,
{
    type Output = T;

    fn count(&self) -> usize {
        todo!()
    }

    fn get(&self, idx: usize) -> Option<Self::Output> {
        (*self).get(idx).copied()
    }

    fn iter(&self) -> impl Iterator<Item = Self::Output> {
        (*self).iter().copied()
    }
}

pub trait FoldInput {
    type VC: Proxy<Output = Vector3F>;
    fn vertices_coords(&self) -> Self::VC;

    type EV: Proxy<Output = Vector2U>;
    fn edges_vertices(&self) -> Self::EV;

    type EA: Proxy<Output = FoldAssignment>;
    fn edges_assignment(&self) -> Self::EA;

    type FV: Proxy<Output = Vector3U>;
    fn faces_vertices(&self) -> Self::FV;
}

pub struct FoldAssignmentParser<'a>(&'a [fold::EdgeAssignment]);

impl FoldAssignmentParser<'_> {
    fn convert(input: fold::EdgeAssignment) -> FoldAssignment {
        match input {
            EdgeAssignment::B => FoldAssignment::Boundary,
            EdgeAssignment::M => FoldAssignment::Mountain,
            EdgeAssignment::V => FoldAssignment::Valley,
            EdgeAssignment::F => FoldAssignment::Facet,
            _ => FoldAssignment::Other,
        }
    }
}

impl Proxy for FoldAssignmentParser<'_> {
    type Output = FoldAssignment;

    fn count(&self) -> usize {
        self.0.len()
    }

    fn get(&self, idx: usize) -> Option<Self::Output> {
        self.0.get(idx).map(|ea| Self::convert(*ea))
    }

    fn iter(&self) -> impl Iterator<Item = Self::Output> {
        self.0.iter().map(|ea| Self::convert(*ea))
    }
}

impl<'a> FoldInput for &'a fold::FrameCore {
    type VC = &'a [Vector3F];

    fn vertices_coords(&self) -> Self::VC {
        let coords: &'a [fold::Vertex] = match &self.vertices.coords {
            Some(coords) => coords.as_slice(),
            None => return &[],
        };

        // Vertex is repr(transparent) so it's ok
        //bytemuck::cast_slice(coords)
        todo!()
    }

    type EV = &'a [Vector2U];

    fn edges_vertices(&self) -> Self::EV {
        let edge_vertices = match &self.edges.vertices {
            Some(indices) => indices.as_slice(),
            None => return &[],
        };

        bytemuck::cast_slice(edge_vertices)
    }

    type EA = FoldAssignmentParser<'a>;

    fn edges_assignment(&self) -> Self::EA {
        FoldAssignmentParser(
            self.edges
                .assignments
                .as_deref()
                .unwrap_or(&[]),
        )
    }

    type FV = &'a [Vector3U];

    fn faces_vertices(&self) -> Self::FV {
        let faces_vertices = match &self.faces.vertices {
            Some(indices) => indices.as_slice(),
            None => return &[],
        };

        //bytemuck::cast_slice(faces_vertices)
        unimplemented!()
    }
}
