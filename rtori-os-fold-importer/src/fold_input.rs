use fold::EdgeAssignment;

pub type Vector3F = [f32; 3];
pub type Vector3U = [u32; 3];
pub type Vector2U = [u32; 2];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FoldAssignment {
    Valley,
    Mountain,
    Facet,
    Other,
}

pub trait Proxy<'a> {
    type Output;
    fn count(&self) -> usize;
    fn get(&self, idx: usize) -> Option<Self::Output>;

    type Iter: Iterator<Item = Self::Output> + 'a
    where
        Self: 'a;
    fn iter(&self) -> Self::Iter;
}

impl<'a, T> Proxy<'a> for &'a [T]
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

    type Iter = std::iter::Copied<std::slice::Iter<'a, T>>;

    fn iter<'b>(&'b self) -> Self::Iter {
        (*self).iter().copied()
    }
}

pub struct FoldAssignmentParser<'a>(&'a [fold::EdgeAssignment]);

impl FoldAssignmentParser<'_> {
    fn convert(input: fold::EdgeAssignment) -> FoldAssignment {
        match input {
            EdgeAssignment::M => FoldAssignment::Mountain,
            EdgeAssignment::V => FoldAssignment::Valley,
            EdgeAssignment::F => FoldAssignment::Facet,
            _ => FoldAssignment::Other,
        }
    }
}

impl<'a> Proxy<'a> for FoldAssignmentParser<'a> {
    type Output = FoldAssignment;

    fn count(&self) -> usize {
        self.0.len()
    }

    fn get(&self, idx: usize) -> Option<Self::Output> {
        self.0.get(idx).map(|ea| Self::convert(*ea))
    }

    type Iter
        = impl Iterator<Item = Self::Output>
    where
        Self: 'a;

    fn iter(&self) -> Self::Iter {
        self.0.iter().map(|ea| Self::convert(*ea))
    }
}

pub trait ImportInput {
    type VerticesCoords<'a>: Proxy<'a, Output = Vector3F>
    where
        Self: 'a;
    fn vertices_coords<'a>(&'a self) -> Self::VerticesCoords<'a>;

    type VerticesFaces<'a>: Proxy<'a, Output = &'a [u32]>
    where
        Self: 'a;
    fn vertices_faces<'a>(&'a self) -> Self::VerticesFaces<'a>;

    type EdgeVertices<'a>: Proxy<'a, Output = Vector2U>
    where
        Self: 'a;
    fn edges_vertices<'a>(&'a self) -> Self::EdgeVertices<'a>;

    type EdgeFaces<'a>: Proxy<'a, Output = &'a [u32]>
    where
        Self: 'a;
    fn edges_faces<'a>(&'a self) -> Self::EdgeFaces<'a>;

    type EdgeAssignment<'a>: Proxy<'a, Output = FoldAssignment>
    where
        Self: 'a;
    fn edges_assignment<'a>(&'a self) -> Self::EdgeAssignment<'a>;

    type EdgeAxialStiffnesses<'a>: Proxy<'a, Output = f32>
    where
        Self: 'a;
    fn edges_axial_stiffnesses<'a>(&'a self) -> Option<Self::EdgeAxialStiffnesses<'a>>;

    type EdgeCreaseStiffnesses<'a>: Proxy<'a, Output = f32>
    where
        Self: 'a;
    fn edges_crease_stiffnesses<'a>(&'a self) -> Option<Self::EdgeCreaseStiffnesses<'a>>;

    type EdgeFoldAngles<'a>: Proxy<'a, Output = f32>
    where
        Self: 'a;
    fn edges_fold_angles<'a>(&'a self) -> Option<Self::EdgeFoldAngles<'a>>;

    type FaceVertices<'a>: Proxy<'a, Output = Vector3U>
    where
        Self: 'a;
    fn faces_vertices<'a>(&'a self) -> Self::FaceVertices<'a>;
}

macro_rules! subclass {
    {
        @trait_decl
        $method:ident -> (req, $associated:tt)
    } => {
        fn $method<'a>(&'a self) -> Self::$associated<'a>;
    };

    {
        @trait_decl
        $method:ident -> (opt, $associated:tt)
    } => {
        fn $method<'a>(&'a self) -> Option<Self::$associated<'a>>;
    };

    {
        @trait_def
        $method:ident -> (req, $associated:tt)
    } => {
        fn $method<'a>(&'a self) -> Self::$associated<'a>{
            ImportInput::$method(self)
        }
    };

    {
        @trait_def
        $method:ident -> (opt, $associated:tt)
    } => {
        fn $method<'a>(&'a self) -> Option<Self::$associated<'a>>{
            ImportInput::$method(self)
        }
    };

    {
        $subclass_name:ident {
            $(
                $method:ident -> ($mode:tt, $associated:tt, $($ty:tt)+);
            )*
        }
    } => {
        trait $subclass_name {
            $(
                type $associated<'a>: Proxy<'a, Output=$($ty)+>
                    where Self: 'a;
                $crate::fold_input::subclass!{@trait_decl $method -> ($mode, $associated)}
            )+
        }

        impl<T> $subclass_name for T where T: ImportInput {
            $(
                type $associated<'a> = <T as ImportInput>::$associated<'a> where T: 'a;
                $crate::fold_input::subclass!{@trait_def $method -> ($mode, $associated)}
            )+
        }
    };
}
pub(crate) use subclass;

/*
impl<'a> FoldInput for &'a fold::FrameCore {
    type VerticesCoords = &'a [Vector3F];

    fn vertices_coords(&self) -> Self::VerticesCoords {
        let coords: &'a [fold::Vertex] = match &self.vertices.coords {
            Some(coords) => coords.as_slice(),
            None => return &[],
        };

        // Vertex is repr(transparent) so it's ok
        //bytemuck::cast_slice(coords)
        todo!()
    }

    type EdgeVertices = &'a [Vector2U];

    fn edges_vertices(&self) -> Self::EdgeVertices {
        let edge_vertices = match &self.edges.vertices {
            Some(indices) => indices.as_slice(),
            None => return &[],
        };

        bytemuck::cast_slice(edge_vertices)
    }

    type EdgeAssignment = FoldAssignmentParser<'a>;

    fn edges_assignment(&self) -> Self::EdgeAssignment {
        FoldAssignmentParser(
            self.edges
                .assignments
                .as_deref()
                .unwrap_or(&[]),
        )
    }

    type FaceVertices = &'a [Vector3U];

    fn faces_vertices(&self) -> Self::FaceVertices {
        let faces_vertices = match &self.faces.vertices {
            Some(indices) => indices.as_slice(),
            None => return &[],
        };

        //bytemuck::cast_slice(faces_vertices)
        unimplemented!()
    }
}
*/
