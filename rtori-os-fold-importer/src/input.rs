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

    type Iter = core::iter::Copied<core::slice::Iter<'a, T>>;

    fn iter<'b>(&'b self) -> Self::Iter {
        (*self).iter().copied()
    }
}

pub trait ImportInput {
    type VerticesCoords<'a>: Proxy<'a, Output = Vector3F>
    where
        Self: 'a;
    fn vertices_coords<'call, 'output>(&'call self) -> Self::VerticesCoords<'output>
    where
        'call: 'output;

    type VerticesEdges<'a>: Proxy<'a, Output = &'a [u32]>
    where
        Self: 'a;
    fn vertices_edges<'call, 'output>(&'call self) -> Self::VerticesEdges<'output>
    where
        'call: 'output;

    type VerticesFaces<'a>: Proxy<'a, Output = &'a [u32]>
    where
        Self: 'a;
    fn vertices_faces<'call, 'output>(&'call self) -> Self::VerticesFaces<'output>
    where
        'call: 'output;

    type EdgeVertices<'a>: Proxy<'a, Output = Vector2U>
    where
        Self: 'a;
    fn edges_vertices<'call, 'output>(&'call self) -> Self::EdgeVertices<'output>
    where
        'call: 'output;

    type EdgeFaces<'a>: Proxy<'a, Output = &'a [u32]>
    where
        Self: 'a;
    fn edges_faces<'call, 'output>(&'call self) -> Self::EdgeFaces<'output>
    where
        'call: 'output;

    type EdgeAssignment<'a>: Proxy<'a, Output = FoldAssignment>
    where
        Self: 'a;
    fn edges_assignment<'call, 'output>(&'call self) -> Self::EdgeAssignment<'output>
    where
        'call: 'output;

    type EdgeAxialStiffnesses<'a>: Proxy<'a, Output = Option<f32>>
    where
        Self: 'a;
    fn edges_axial_stiffnesses<'call, 'output>(
        &'call self,
    ) -> Option<Self::EdgeAxialStiffnesses<'output>>
    where
        'call: 'output;

    type EdgeCreaseStiffnesses<'a>: Proxy<'a, Output = Option<f32>>
    where
        Self: 'a;
    fn edges_crease_stiffnesses<'call, 'output>(
        &'call self,
    ) -> Option<Self::EdgeCreaseStiffnesses<'output>>
    where
        'call: 'output;

    type EdgeFoldAngles<'a>: Proxy<'a, Output = f32>
    where
        Self: 'a;
    fn edges_fold_angles<'call, 'output>(&'call self) -> Option<Self::EdgeFoldAngles<'output>>
    where
        'call: 'output;

    type FaceVertices<'a>: Proxy<'a, Output = Vector3U>
    where
        Self: 'a;
    fn faces_vertices<'call, 'output>(&'call self) -> Self::FaceVertices<'output>
    where
        'call: 'output;
}

macro_rules! subclass {
    {
        @trait_decl
        $method:ident -> (req, $associated:tt)
    } => {
        fn $method<'call, 'output>(&'call self) -> Self::$associated<'output> where 'call: 'output;
    };

    {
        @trait_decl
        $method:ident -> (opt, $associated:tt)
    } => {
        fn $method<'call, 'output>(&'call self) -> Option<Self::$associated<'output>> where 'call: 'output;
    };

    {
        @trait_def
        $method:ident -> (req, $associated:tt)
    } => {
        fn $method<'call, 'output>(&'call self) -> Self::$associated<'output> where 'call: 'output{
            $crate::input::ImportInput::$method(self)
        }
    };

    {
        @trait_def
        $method:ident -> (opt, $associated:tt)
    } => {
        fn $method<'call, 'output>(&'call self) -> Option<Self::$associated<'output>> where 'call: 'output{
            $crate::input::ImportInput::$method(self)
        }
    };

    {
        $visibility:vis $subclass_name:ident {
            $(
                $method:ident -> ($mode:tt, $associated:tt, $($ty:tt)+);
            )*
        }
    } => {
        $visibility trait $subclass_name {
            $(
                type $associated<'a>: Proxy<'a, Output=$($ty)+>
                    where Self: 'a;
                $crate::input::subclass!{@trait_decl $method -> ($mode, $associated)}
            )+
        }

        impl<T> $subclass_name for T where T: ImportInput {
            $(
                type $associated<'a> = <T as ImportInput>::$associated<'a> where T: 'a;
                $crate::input::subclass!{@trait_def $method -> ($mode, $associated)}
            )+
        }
    };
}
pub(crate) use subclass;
