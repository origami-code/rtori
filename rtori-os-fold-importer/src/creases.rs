pub type CreaseIndex = u32;

#[derive(Debug, Clone, Copy)]
pub struct CreaseFace {
    pub face_index: u32,
    pub complement_vertex_index: u32,
}

#[derive(Debug, Clone, Copy)]
pub struct Crease {
    pub faces: [CreaseFace; 2],
    pub edge_index: u32,
    pub fold_angle: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExtractCreasesError {
    EdgeVerticesMissing,
    EdgeFacesMissing,
    EdgeAssignmentsMissing,
    FaceVerticesMissing,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExtractCreasesIteratorErrorKind {
    /// edge #{edge_index} has {face_count} faces, which is invalid
    EdgeHasInvalidNumberOfFaces { face_count: usize },
    /// edge's face #{face_number} (index {face_index}) is not triangular (it has {vertices_count} vertices)
    NonTriangularFace {
        face_number: usize,
        face_index: usize,
        vertices_count: usize,
    },
    /// edge #{edge_index}'s face #{face_number} (index {face_index}) does not contain both vertices of the edge
    InvalidFaceVertices {
        face_number: usize,
        face_index: usize,
    },
    /// edge #{edge_index}'s face #{face_number} (index {face_index}) contains at least twice same vertex index
    FaceHasTwiceTheSameVertex {
        face_number: usize,
        face_index: usize,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ExtractCreasesIteratorError {
    pub edge_index: usize,
    pub kind: ExtractCreasesIteratorErrorKind,
}

impl core::fmt::Display for ExtractCreasesIteratorError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let edge_index = self.edge_index;
        match self.kind {
            ExtractCreasesIteratorErrorKind::EdgeHasInvalidNumberOfFaces { face_count }
                => write!(f, "edge #{edge_index} has {face_count} faces, which is invalid"),
            ExtractCreasesIteratorErrorKind::NonTriangularFace { face_number, face_index, vertices_count }
                => write!(f, "edge #{edge_index}'s face #{face_number} (index {face_index}) is not triangular (it has {vertices_count} vertices)"),
            ExtractCreasesIteratorErrorKind::InvalidFaceVertices { face_number, face_index }
                => write!(f, "edge #{edge_index}'s face #{face_number} (index {face_index}) does not contain both vertices of the edge"),
            ExtractCreasesIteratorErrorKind::FaceHasTwiceTheSameVertex { face_number, face_index }
                => write!(f, "edge #{edge_index}'s face #{face_number} (index {face_index}) contains at least twice same vertex index")
        }
    }
}

impl core::error::Error for ExtractCreasesIteratorError {}

use crate::{
    input::{FoldAssignment, Proxy, Vector2U, Vector3U},
    ImportInput,
};

crate::input::subclass! {
    pub ExtractCreasesInput {
        edges_vertices -> (req, EdgeVertices, Vector2U);
        edges_faces -> (req, EdgeFaces, &'a [u32]);
        edges_assignment -> (req, EdgeAssignment, FoldAssignment);
        edges_fold_angles -> (opt, EdgeFoldAngles, Option<f32>);
        faces_vertices -> (req, FaceVertices, Vector3U);
    }
}

struct RelevantEdgeInfo<'a> {
    /// Which vertices, by index, compose this edge
    vertex_indices: Vector2U,

    /// The faces, by index, neighbouring this edge
    face_indices: &'a [u32],

    /// The assignment of the given crease
    /// Mountain, Valley, Facet ?
    assignment: FoldAssignment,

    /// Fold angle if specified, otherwise `Option::None`
    fold_angle: Option<f32>,
}

/// Iterate through the edges and returns a subset of its elements
fn iter_edges<'a, FI: ExtractCreasesInput>(
    fi: &'a FI,
) -> impl Iterator<Item = RelevantEdgeInfo<'a>> + use<'a, FI> {
    let edges_vertices = fi.edges_vertices();
    let edges_faces = fi.edges_faces();
    let edges_assignment = fi.edges_assignment();
    let edges_fold_angles = fi.edges_fold_angles();

    let vit = edges_vertices.iter();
    let faces_it = edges_faces.iter();
    let ait = edges_assignment.iter();
    let fit = edges_fold_angles
        .as_ref()
        .map_or_else(
            // No such field
            || itertools::Either::Right(core::iter::repeat(None)),
            // Such a field
            |v| itertools::Either::Left(v.iter().map(|i| i)),
        )
        .into_iter();

    itertools::izip!(vit, faces_it, ait, fit).map(
        |(vertex_indices, face_indices, assignment, fold_angle)| RelevantEdgeInfo {
            vertex_indices,
            face_indices,
            assignment,
            fold_angle,
        },
    )
}

pub fn count_creases<'a, FI: ExtractCreasesInput>(input: &'a FI) -> usize {
    input
        .edges_assignment()
        .iter()
        .filter(|ea| {
            matches!(
                ea,
                FoldAssignment::Facet | FoldAssignment::Mountain | FoldAssignment::Valley
            )
        })
        .count()
}

/// Extract creases from the input
pub fn extract_creases<'a, FI: ExtractCreasesInput>(
    input: &'a FI,
) -> impl Iterator<Item = Result<Crease, ExtractCreasesIteratorError>> + use<'a, FI> {
    let default_mountain_fold_angle = -core::f32::consts::PI;
    let default_valley_fold_angle = core::f32::consts::PI;

    let iterator = iter_edges(input);

    let faces_vertices = input.faces_vertices();

    iterator
        // Filter out the irrelevant folds (non-mountain, valley or facet)
        .enumerate()
        .filter_map(move |(edge_index, edge_info)| {


            match (edge_info.assignment, edge_info.fold_angle) {
                (FoldAssignment::Facet | FoldAssignment::Mountain | FoldAssignment::Valley, Some(a)) => Some(a),
                (FoldAssignment::Mountain, None) => Some(default_mountain_fold_angle),
                (FoldAssignment::Valley, None) => Some(default_valley_fold_angle),
                (FoldAssignment::Facet, None) => Some(0.0),
                _ => None
            }.map(|fold_angle| (
                edge_index,
                edge_info.vertex_indices,
                edge_info.face_indices,
                fold_angle,
            ))
        })
        .map(move |(edge_index, vertex, faces, fold_angle)| {
            if faces.len() < 2 {
                // When an edge is M, V or F (as it should now be), then it must have at least two faces
                return Err(ExtractCreasesIteratorError{
                    edge_index,
                    kind: ExtractCreasesIteratorErrorKind::EdgeHasInvalidNumberOfFaces{
                        face_count: faces.len(),
                    }
                });
            }

            let per_face = |face_number| {
                let face_index = usize::try_from(faces[face_number]).unwrap();
                let indices: Vector3U = faces_vertices.get(face_index).unwrap();
                if indices.len() != 3 {
                    return Err(ExtractCreasesIteratorError{
                        edge_index,
                        kind: ExtractCreasesIteratorErrorKind::NonTriangularFace { face_number, face_index, vertices_count: indices.len() }
                    })
                }

                // Now we know indices countains three elements
                // We find which one of the face's vertices isn't on the edge
                let vertex_0_index = indices.iter().position(|face_vertex_index| *face_vertex_index == vertex[0]);
                let vertex_1_index = indices.iter().position(|face_vertex_index| *face_vertex_index == vertex[1]);
                let (v0_idx, v1_idx) = match (vertex_0_index, vertex_1_index) {
                    (Some(a), Some(b)) => (a,b),
                    _ => return Err(ExtractCreasesIteratorError{
                        edge_index,
                        kind: ExtractCreasesIteratorErrorKind::InvalidFaceVertices { face_number, face_index }
                    }),
                };

                if v0_idx == v1_idx {
                    return Err(ExtractCreasesIteratorError{
                        edge_index,
                        kind: ExtractCreasesIteratorErrorKind::FaceHasTwiceTheSameVertex { face_number, face_index }
                    });
                }

                let complement_vertex_index_index = [0, 1, 2]
                    .into_iter()
                    .find(|face_vertex_index| {
                        let face_vertex_index = usize::try_from(*face_vertex_index).unwrap();
                        face_vertex_index != v0_idx && face_vertex_index != v1_idx
                    })
                    .expect("Should find the complement given we checked every conceivable reason it wouldn't give it");

                let complement_vertex_index = indices[complement_vertex_index_index];

                let result = CreaseFace {
                    face_index: face_index as u32,
                    complement_vertex_index
                };

                let flip = (v1_idx == 1 + v0_idx) || (v0_idx == v1_idx + 2);

                Ok((result, flip))
            };

            let (face_0, _) = per_face(0)?;
            let (face_1, flip) = per_face(1)?;
            let face_parameters = if flip {
                [face_1, face_0]
            } else {
                [face_0, face_1]
            };


            Ok(Crease {
                faces: face_parameters,
                edge_index: edge_index as u32,
                fold_angle
            })
        })
}
