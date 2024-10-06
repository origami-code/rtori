use rtori_core::fold_importer::transform;

/// A transformed fold (opaque)
/// cbindgen:ignore
pub struct TransformedData<'alloc> {
    input: crate::Arc<'alloc, super::FoldFile<'alloc>>,
    frame: u16,
    transform: transform::TransformedData<crate::ContextAllocator<'alloc>>,
}

pub unsafe extern "C" fn rtori_fold_transform<'alloc>(
    fold: *const super::FoldFile<'alloc>,
    frame_index: u16,
) -> *mut TransformedData<'alloc> {
    let alloc = unsafe { &*fold }.ctx.allocator;
    let fold = unsafe { crate::Arc::from_raw_in(fold, alloc) };

    let transform = {
        let frame = fold.parsed.frame(frame_index).unwrap();
        let frame = frame.get();

        let transform = transform::transform_in(&frame, alloc).unwrap();

        transform
    };

    let result = TransformedData {
        input: fold,
        frame: frame_index,
        transform,
    };

    let result_boxed = Box::new_in(result, alloc);
    Box::into_raw(result_boxed)
}

pub unsafe extern "C" fn rtori_fold_transformed_drop<'alloc>(
    transform: *mut TransformedData<'alloc>,
) {
    let alloc = unsafe { &*transform }.input.ctx.allocator;
    let boxed = unsafe { Box::from_raw_in(transform, alloc) };
    drop(boxed);
}

#[derive(Copy, Clone)]
#[repr(C)]
pub enum TransformedQuery {
    /// Outputs the transformed frame's amount of edges
    /// Implies the use of a [`QueryOutput::u32_output`] as data parameter
    EdgesCount,

    /// Outputs the transformed frame's amount of faces
    /// Implies the use of a [`QueryOutput::u32_output`] as data parameter
    FacesCount,

    /// Outputs the transformed frame's `faces_vertices`
    /// Implies the use of a [`QueryOutput::vec3u_array_output`] as data parameter
    FacesVertexIndices,

    /// Outputs the transformed frame's `rtori:faces_uvs`
    /// Implies the use of a [`QueryOutput::vec3u_array_output`] as data parameter
    FacesUVs,
}

/// SAFETY: calling this is threadsafe over other `fold_transformed_*` operations, except `fold_transformed_drop`
#[no_mangle]
pub unsafe extern "C" fn rtori_fold_transformed_query<'alloc>(
    transformed: *const TransformedData<'alloc>,
    query: TransformedQuery,
    mut output: core::ptr::NonNull<crate::QueryOutput>,
) -> super::FoldOperationStatus {
    let transformed = unsafe { &*transformed };

    match query {
        field @ (TransformedQuery::EdgesCount | TransformedQuery::FacesCount) => {
            let count = u32::try_from(match field {
                TransformedQuery::EdgesCount => transformed.transform.edges_faces.len(),
                TransformedQuery::FacesCount => {
                    transformed.transform.triangulated.face_indices.len()
                }
                _ => unreachable!(),
            })
            .unwrap();

            // SAFETY: output must be pointing to valid memory
            unsafe { output.as_mut().u32_output.as_uninit_mut() }.write(count);
            super::FoldOperationStatus::Success
        }
        TransformedQuery::FacesVertexIndices => {
            let source = transformed.transform.triangulated.face_indices.as_slice();
            unsafe { output.as_mut().copy_vec3u(Some(source)) };
            super::FoldOperationStatus::Success
        }
        TransformedQuery::FacesUVs => {
            let frame = transformed.input.parsed.frame(transformed.frame).unwrap();
            let frame = frame.get();

            let original_face_indices = frame.faces.vertices.as_ref();
            let original_face_uvs = frame.faces.uvs.as_ref();

            let source = if (original_face_indices.is_none() || original_face_uvs.is_none()) {
                None
            } else {
                let original_face_indices = original_face_indices.unwrap();
                let original_face_uvs = original_face_uvs.unwrap();

                Some(transformed.transform.triangulated.iter_faces().map(
                    |(transformed_face_indices, previous_face_index)| {
                        let original_face_indices =
                            &original_face_indices[previous_face_index as usize];
                        let original_uvs = &original_face_uvs[previous_face_index as usize];

                        transformed_face_indices.map(|vertex_index_needle| {
                            let vertex_number_in_original = original_face_indices
                                .0
                                .iter()
                                .position(|vertex_index_candidate| {
                                    *vertex_index_candidate == vertex_index_needle
                                })
                                .unwrap();

                            let uv_index = original_uvs[vertex_number_in_original];

                            uv_index
                        })
                    },
                ))
            };

            match source {
                Some(source) => unsafe { output.as_mut().extend_vec3u(source) },
                None => unsafe { output.as_mut().empty_vec3u() },
            }

            super::FoldOperationStatus::Success
        }
    }
}
