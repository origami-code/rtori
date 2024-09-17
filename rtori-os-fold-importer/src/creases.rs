
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
    EdgeHasInvalidNumberOfFaces { face_count: usize },
    NonTriangularFace { vertices_count: usize },
    // edge[x] says it is connected to face[y] but the vertices of edge[x] aren't both in face[y]
    InvalidFaceVertices { face_index: usize },
    FaceHasTwiceTheSameVertex { face_index: usize },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ExtractCreasesIteratorError {
    pub edge_index: usize,
    pub kind: ExtractCreasesIteratorErrorKind,
}

/// foldAngles is not provided,
pub fn extract_creases(
    input: &fold::FrameCore,
) -> Result<
    impl Iterator<Item = Result<Crease, ExtractCreasesIteratorError>> + use<'_>,
    ExtractCreasesError,
> {
    let default_mountain_fold_angle = -180.0;
    let default_valley_fold_angle = 180.0;

    if input.edges.vertices.is_none() {
        return Err(ExtractCreasesError::EdgeVerticesMissing);
    } else if input.edges.faces.is_none() {
        return Err(ExtractCreasesError::EdgeFacesMissing);
    } else if input.edges.assignments.is_none() {
        return Err(ExtractCreasesError::EdgeAssignmentsMissing);
    } else if input.faces.vertices.is_none() {
        return Err(ExtractCreasesError::FaceVerticesMissing);
    }

    let iterator = fold::iter!(
        &input.edges,
        required(vertices, faces, assignments),
        optional(fold_angles)
    )
    .expect("required fields have already been checked");

    let faces_vertices = input.faces.vertices.as_ref().unwrap().as_slice();

    let processed = iterator
        // Filter out the irrelevant folds (non-mountain, valley or facet)
        .filter_map(move |(vertex, faces, assignment, fold_angles)| {
            match (assignment, fold_angles) {
                (fold::EdgeAssignment::M | fold::EdgeAssignment::V | fold::EdgeAssignment::F, Some(a)) => Some(*a),
                (fold::EdgeAssignment::M, None) => Some(default_mountain_fold_angle),
                (fold::EdgeAssignment::V, None) => Some(default_valley_fold_angle),
                (fold::EdgeAssignment::F, None) => Some(0.0),
                _ => None
            }.map(|fold_angle| (
                vertex,
                faces,
                fold_angle,
            ))
        })
        .enumerate()
        .map(move |(edge_index, (vertex, faces, fold_angle))| {
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
                let indices = faces_vertices[face_index].0.as_slice();
                if indices.len() != 3 {
                    return Err(ExtractCreasesIteratorError{
                        edge_index,
                        kind: ExtractCreasesIteratorErrorKind::NonTriangularFace { vertices_count: indices.len() }
                    })
                }

                // Now we know indices countains three elements
                // We find which one of the face's vertices isn't on the edge
                let vertex_0_index = indices.iter().position(|face_vertex_index| *face_vertex_index == vertex.0[0]);
                let vertex_1_index = indices.iter().position(|face_vertex_index| *face_vertex_index == vertex.0[1]);
                let (v0_idx, v1_idx) = match (vertex_0_index, vertex_1_index) {
                    (Some(a), Some(b)) => (a,b),
                    _ => return Err(ExtractCreasesIteratorError{
                        edge_index,
                        kind: ExtractCreasesIteratorErrorKind::InvalidFaceVertices { face_index }
                    }),
                };

                if v0_idx == v1_idx {
                    return Err(ExtractCreasesIteratorError{
                        edge_index,
                        kind: ExtractCreasesIteratorErrorKind::FaceHasTwiceTheSameVertex { face_index }
                    });
                }

                let complement_vertex_index = *indices
                    .iter()
                    .find(|face_vertex_index| usize::try_from(**face_vertex_index).unwrap() != v0_idx && usize::try_from(**face_vertex_index).unwrap() != v1_idx)
                    .expect("Should find the complement given we checked every conceivable reason it wouldn't give it");

                let result = CreaseFace {
                    face_index: face_index as u32,
                    complement_vertex_index
                };

                let flip = (v1_idx - v0_idx == 1) || (v0_idx == v1_idx + 2);

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
        });

    Ok(processed)
}
