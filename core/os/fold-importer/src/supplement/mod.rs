use core::alloc::Allocator;

extern crate alloc;
use alloc::vec::Vec;
use fold::{EdgeIndex, FaceIndex, VertexIndex};

use crate::{
    input::{FoldAssignment, ImportInput, Proxy, Vector2U, Vector3F, Vector3U},
    triangulation::TriangulatedDiff,
};

/// In FOLD terms, compute `vertices_edges` given `edges_vertices`
fn create_vertices_edges<EdgeVerticesSource, A>(
    edge_vertices: EdgeVerticesSource,
    vertices_count: usize,
    allocator: A,
) -> Vec<Vec<EdgeIndex, A>, A>
where
    EdgeVerticesSource: IntoIterator<Item = [VertexIndex; 2]>,
    A: Allocator + Clone,
{
    let mut vertices_edges = Vec::with_capacity_in(vertices_count, allocator.clone());
    vertices_edges.resize(vertices_count, Vec::new_in(allocator));

    for (edge_index, edge_vertex_indices) in edge_vertices.into_iter().enumerate() {
        for vertex_index in edge_vertex_indices.iter() {
            vertices_edges[*vertex_index as usize].push(edge_index as u32)
        }
    }

    vertices_edges
}

/// In FOLD terms, compute `vertices_faces` given `faces_vertices`
fn create_vertices_faces<'a, FaceVerticesSource, FaceVertices, A>(
    face_vertices: FaceVerticesSource,
    vertices_count: usize,
    allocator: A,
) -> Vec<Vec<FaceIndex, A>, A>
where
    FaceVerticesSource: IntoIterator<Item = FaceVertices>,
    FaceVertices: AsRef<[VertexIndex]>,
    A: Allocator + Clone,
{
    let mut vertices_faces = Vec::with_capacity_in(vertices_count, allocator.clone());
    vertices_faces.resize(vertices_count, Vec::new_in(allocator));

    for (face_index, face_vertices) in face_vertices.into_iter().enumerate() {
        for vertex_index in face_vertices.as_ref().iter() {
            vertices_faces[*vertex_index as usize].push(face_index as u32)
        }
    }

    vertices_faces
}

/// In FOLD terms, compute `edges_faces` given `faces_vertices` and `edges_vertices`
fn create_edges_faces<'a, A: Allocator + Clone>(
    input: &'a fold::FrameCore,
    allocator: A,
) -> Result<Vec<Vec<FaceIndex, A>, A>, ()> {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    struct Key([VertexIndex; 2]);
    impl Key {
        pub const fn new(src: [u32; 2]) -> Self {
            // MAke sure that regardless of the sort order, we're the same key
            let sorted = if src[0] < src[1] {
                [src[0], src[1]]
            } else {
                [src[1], src[0]]
            };
            Self(sorted)
        }
    }

    let mut edge_to_face_map =
        alloc::collections::BTreeMap::<Key, Vec<FaceIndex, A>, A>::new_in(allocator.clone());

    for (face_index, face_vertex_indices) in
        input.faces.vertices.as_ref().unwrap().iter().enumerate()
    {
        let face_order = face_vertex_indices.0.len();
        for i in 0..face_order {
            let u = face_vertex_indices.0[i];
            let v = face_vertex_indices.0[(i + 1) % face_order];
            let k = Key::new([u, v]);

            let faces_for_edge = edge_to_face_map.try_insert(k, Vec::new_in(allocator.clone()));
            match faces_for_edge {
                Ok(inserted) => inserted.push(face_index as u32),
                Err(mut e) => e.entry.get_mut().push(face_index as u32),
            }
        }
    }

    if edge_to_face_map.len() != input.edges.vertices.as_ref().map(|v| v.len()).unwrap() {
        return Err(());
    }

    let mut edges_faces = Vec::with_capacity_in(input.edges.count(), allocator.clone());
    edges_faces.resize(input.edges.count(), Vec::new_in(allocator));

    for (edge_index, edge_vertex_indices) in
        input.edges.vertices.as_ref().unwrap().iter().enumerate()
    {
        let u = edge_vertex_indices.0[0];
        let v = edge_vertex_indices.0[1];
        let to_query = Key::new([u, v]);

        let faces = edge_to_face_map.remove_entry(&to_query).unwrap();
        // What to do with edges without faces ?

        edges_faces[edge_index] = faces.1;
    }

    Ok(edges_faces)
}

#[derive(Debug, Clone, PartialEq)]
pub enum TransformError {
    MissingRequiredField(fold::Field),
    TriangulationError(crate::triangulation::Triangulate3DError),
    IncorrectInput,
}

/// The `FoldSupplement` is the additional information to the Fold Input,
/// needed to load an Origami Simulator solver.
pub struct FoldSupplement<A>
where
    A: Allocator,
{
    pub triangulated: crate::triangulation::TriangulatedDiff<A>,
    pub vertices_edges: Vec<Vec<EdgeIndex, A>, A>,
    pub vertices_faces: Vec<Vec<FaceIndex, A>, A>,
    pub edges_faces: Vec<Vec<FaceIndex, A>, A>,
}

impl<A> FoldSupplement<A>
where
    A: Allocator,
{
    pub const fn with_fold<'frame>(
        &'frame self,
        frame: &'frame fold::FrameCore,
    ) -> SupplementedInput<'frame, A> {
        SupplementedInput::new(frame, self)
    }
}

/// The `SupplementedInput` combines the fold source and the `FoldSupplement` to provide
/// the required information for the processing and eventual loading of the Fold into an Origami Simulator solver.
pub struct SupplementedInput<'frame, A>
where
    A: Allocator,
{
    pub source: &'frame fold::FrameCore<'frame>,
    pub transformed: &'frame FoldSupplement<A>,
}

impl<'frame, A> SupplementedInput<'frame, A>
where
    A: Allocator,
{
    pub const fn new(
        fold: &'frame fold::FrameCore,
        transformed: &'frame FoldSupplement<A>,
    ) -> Self {
        {
            Self {
                transformed,
                source: fold,
            }
        }
    }
}
pub struct VerticesCoords<'input, A>(&'input SupplementedInput<'input, A>)
where
    A: Allocator;

impl<'input, A> crate::input::Proxy<'input> for VerticesCoords<'input, A>
where
    A: Allocator,
{
    type Output = [f32; 3];

    fn count(&self) -> usize {
        self.0.source.vertices.count()
    }

    fn get(&self, idx: usize) -> Option<Self::Output> {
        self.0
            .source
            .vertices
            .coords
            .as_ref()
            .and_then(|v| v.get(idx))
            .map(|slice| {
                if slice.len() != 3 {
                    panic!();
                }

                [slice[0], slice[1], slice[2]]
            })
    }

    type Iter = impl core::iter::ExactSizeIterator<Item = Self::Output>;

    fn iter(&self) -> Self::Iter {
        self.0
            .source
            .vertices
            .coords
            .as_ref()
            .unwrap()
            .iter()
            .map(|slice| {
                if slice.len() != 3 {
                    panic!();
                }

                [slice[0], slice[1], slice[2]]
            })
    }
}

struct VecWrapper<'input, T, A>(&'input [Vec<T, A>])
where
    A: Allocator;

impl<'input, T, A> crate::input::Proxy<'input> for VecWrapper<'input, T, A>
where
    A: Allocator,
{
    type Output = &'input [T];

    fn count(&self) -> usize {
        self.0.len()
    }

    fn get(&self, idx: usize) -> Option<Self::Output> {
        self.0.get(idx).map(|v| v.as_slice())
    }

    type Iter = impl core::iter::ExactSizeIterator<Item = Self::Output>;

    fn iter(&self) -> Self::Iter {
        self.0.iter().map(|v| v.as_slice())
    }
}

const fn convert(ea: fold::EdgeAssignment) -> crate::input::FoldAssignment {
    match ea {
        fold::EdgeAssignment::M => crate::input::FoldAssignment::Mountain,
        fold::EdgeAssignment::V => crate::input::FoldAssignment::Valley,
        fold::EdgeAssignment::F => crate::input::FoldAssignment::Facet,
        _ => crate::input::FoldAssignment::Other,
    }
}

pub struct TranslatingProxy<'a>(&'a [fold::EdgeAssignment]);
impl<'a> Proxy<'a> for TranslatingProxy<'a> {
    type Output = crate::input::FoldAssignment;

    fn count(&self) -> usize {
        self.0.len()
    }

    fn get(&self, idx: usize) -> Option<Self::Output> {
        self.0.get(idx).map(|ea| convert(*ea))
    }

    type Iter
        = impl ExactSizeIterator<Item = Self::Output>
    where
        Self: 'a;

    fn iter(&self) -> Self::Iter {
        self.0.iter().map(|ea| convert(*ea))
    }
}

pub struct DegreesToRadiansProxy<'a>(&'a [Option<f32>]);
impl<'a> DegreesToRadiansProxy<'a> {
    const DEGREES_TO_RADIANS_FACTOR: f32 = core::f32::consts::PI / 180.0f32;
}
impl<'a> Proxy<'a> for DegreesToRadiansProxy<'a> {
    type Output = Option<f32>;

    fn count(&self) -> usize {
        self.0.len()
    }

    fn get(&self, idx: usize) -> Option<Self::Output> {
        self.0
            .get(idx)
            .map(|value| value.map(|val| val * Self::DEGREES_TO_RADIANS_FACTOR))
    }

    type Iter
        = impl ExactSizeIterator<Item = Self::Output>
    where
        Self: 'a;

    fn iter(&self) -> Self::Iter {
        self.0
            .iter()
            .map(|value| value.map(|val| val * Self::DEGREES_TO_RADIANS_FACTOR))
    }
}

impl<'input, A> crate::input::ImportInput for SupplementedInput<'input, A>
where
    A: core::alloc::Allocator,
{
    type VerticesCoords<'a>
        = VerticesCoords<'a, A>
    where
        Self: 'a;

    fn vertices_coords<'call, 'output>(&'call self) -> Self::VerticesCoords<'output>
    where
        'call: 'output,
    {
        VerticesCoords(self)
    }

    type VerticesEdges<'a>
        = impl crate::input::Proxy<'a, Output = &'a [EdgeIndex]>
    where
        Self: 'a;

    fn vertices_edges<'call, 'output>(&'call self) -> Self::VerticesEdges<'output>
    where
        'call: 'output,
    {
        VecWrapper(self.transformed.vertices_edges.as_slice())
    }

    type VerticesFaces<'a>
        = impl crate::input::Proxy<'a, Output = &'a [FaceIndex]>
    where
        Self: 'a;

    fn vertices_faces<'call, 'output>(&'call self) -> Self::VerticesFaces<'output>
    where
        'call: 'output,
    {
        VecWrapper(self.transformed.vertices_faces.as_slice())
    }

    type EdgeVertices<'a>
        = impl crate::input::Proxy<'a, Output = [VertexIndex; 2]>
    where
        Self: 'a;

    fn edges_vertices<'call, 'output>(&'call self) -> Self::EdgeVertices<'output>
    where
        'call: 'output,
    {
        struct Source<'a>(&'a [fold::EdgeVertexIndices]);
        impl<'a> Proxy<'a> for Source<'a> {
            type Output = [VertexIndex; 2];

            fn count(&self) -> usize {
                self.0.len()
            }

            fn get(&self, idx: usize) -> Option<Self::Output> {
                self.0.get(idx).map(|v| v.0)
            }

            type Iter
                = impl ExactSizeIterator<Item = Self::Output>
            where
                Self: 'a;

            fn iter(&self) -> Self::Iter {
                self.0.iter().map(|inner| inner.0)
            }
        }

        self.source
            .edges
            .vertices
            .as_ref()
            .map(|v| Source(v.as_slice()))
            .unwrap()
    }

    type EdgeFaces<'a>
        = impl crate::input::Proxy<'a, Output = &'a [FaceIndex]>
    where
        Self: 'a;

    fn edges_faces<'call, 'output>(&'call self) -> Self::EdgeFaces<'output>
    where
        'call: 'output,
    {
        VecWrapper(self.transformed.edges_faces.as_slice())
    }

    type EdgeAssignment<'a>
        = TranslatingProxy<'a>
    where
        Self: 'a;

    fn edges_assignment<'call, 'output>(&'call self) -> Self::EdgeAssignment<'output>
    where
        'call: 'output,
    {
        self.source
            .edges
            .assignments
            .as_ref()
            .map(|v| TranslatingProxy(v.as_slice()))
            .unwrap()
    }

    type EdgeAxialStiffnesses<'a>
        = &'a [Option<f32>]
    where
        Self: 'a;

    fn edges_axial_stiffnesses<'call, 'output>(
        &'call self,
    ) -> Option<Self::EdgeAxialStiffnesses<'output>>
    where
        'call: 'output,
    {
        self.source
            .edges
            .axial_stiffness
            .as_ref()
            .map(|v| v.as_slice())
    }

    type EdgeCreaseStiffnesses<'a>
        = &'a [Option<f32>]
    where
        Self: 'a;

    fn edges_crease_stiffnesses<'call, 'output>(
        &'call self,
    ) -> Option<Self::EdgeCreaseStiffnesses<'output>>
    where
        'call: 'output,
    {
        self.source
            .edges
            .crease_stiffness
            .as_ref()
            .map(|v| v.as_slice())
    }

    type EdgeFoldAngles<'a>
        = DegreesToRadiansProxy<'a>
    where
        Self: 'a;

    fn edges_fold_angles<'call, 'output>(&'call self) -> Option<Self::EdgeFoldAngles<'output>>
    where
        'call: 'output,
    {
        self.source
            .edges
            .fold_angles
            .as_ref()
            .map(|v| DegreesToRadiansProxy(v))
    }

    type FaceVertices<'a>
        = impl crate::input::Proxy<'a, Output = [VertexIndex; 3]>
    where
        Self: 'a;

    fn faces_vertices<'call, 'output>(&'call self) -> Self::FaceVertices<'output>
    where
        'call: 'output,
    {
        struct TriangulatedProxy<'a, A>(&'a TriangulatedDiff<A>)
        where
            A: Allocator;
        impl<'a, A> Proxy<'a> for TriangulatedProxy<'a, A>
        where
            A: Allocator,
        {
            type Output = [VertexIndex; 3];

            fn count(&self) -> usize {
                self.0.face_indices.len()
            }

            fn get(&self, idx: usize) -> Option<Self::Output> {
                self.0.face_indices.get(idx).copied()
            }

            type Iter
                = impl ExactSizeIterator<Item = Self::Output>
            where
                Self: 'a;

            fn iter(&self) -> Self::Iter {
                self.0.face_indices.iter().copied()
            }
        }

        TriangulatedProxy(&self.transformed.triangulated)
    }
}

pub fn transform_in<A: Allocator + Clone>(
    input: &fold::FrameCore,
    allocator: A,
) -> Result<FoldSupplement<A>, TransformError> {
    // First, triangulate
    let triangulated = crate::triangulation::triangulate3d_collect(
        input
            .faces
            .vertices
            .as_ref()
            .ok_or(TransformError::MissingRequiredField(
                fold::Field::FacesVertices,
            ))?,
        input
            .vertices
            .coords
            .as_ref()
            .ok_or(TransformError::MissingRequiredField(
                fold::Field::VerticesCoords,
            ))?,
        allocator.clone(),
    )
    .map_err(|e| TransformError::TriangulationError(e))?;

    transform_triangulated_in(input, triangulated, allocator)
}

pub fn transform_triangulated_in<A: Allocator + Clone>(
    input: &fold::FrameCore,
    triangulated: crate::triangulation::TriangulatedDiff<A>,
    allocator: A,
) -> Result<FoldSupplement<A>, TransformError> {
    // Then, we compute the required mappings
    let vertices_count = input.vertices.count();
    let vertices_edges = create_vertices_edges(
        core::iter::chain(
            input
                .edges
                .vertices
                .as_ref()
                .unwrap()
                .iter()
                .map(|wrapped| wrapped.0),
            triangulated.additional_edges.iter().copied(),
        ),
        vertices_count,
        allocator.clone(),
    );
    let vertices_faces = create_vertices_faces(
        triangulated.face_indices.iter(),
        vertices_count,
        allocator.clone(),
    );
    let edges_faces =
        create_edges_faces(input, allocator.clone()).map_err(|_| TransformError::IncorrectInput)?;

    Ok(FoldSupplement {
        triangulated,
        vertices_edges,
        vertices_faces,
        edges_faces,
    })
}
