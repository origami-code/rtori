type VertexIndex = u32;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Triangulate3DError {
    ErrFaceIsNotAPolygon { vertex_count: usize },
    ErrVertexIsNot3D { vertex_index: usize },
}

/// Operates on a single face at a time
/// The two callbacks (`replace_face_partial` and `append_edge`) are called to create the new model.
/// - `replace_face_partial` is called for every face created and/or reused (it's a replace operation)
/// - `append_edge` is called only for new edges as it's not necessary to ever remove edges (it's an append operation)
#[inline]
pub fn triangulate3d<'a, Vertex, FuncRF, FuncAE>(
    face_vertex_indices: &'a [VertexIndex],
    vertices: &'a [Vertex],
    mut replace_face_partial: FuncRF,
    mut append_edge: FuncAE,
) -> Result<(), Triangulate3DError>
where
    Vertex: core::ops::Deref<Target = [f32]>,
    FuncRF: FnMut([VertexIndex; 3]),
    FuncAE: FnMut([VertexIndex; 2]),
{
    let vertex_count = face_vertex_indices.len();

    let mut register_face = |numbers: [u32; 3]| {
        let original_face = face_vertex_indices;
        let new_face = [
            original_face[numbers[0] as usize],
            original_face[numbers[1] as usize],
            original_face[numbers[2] as usize],
        ];
        replace_face_partial(new_face);
    };

    if vertex_count < 3 {
        Err(Triangulate3DError::ErrFaceIsNotAPolygon { vertex_count })
    } else if vertex_count == 3 {
        register_face([0, 1, 2]);
        Ok(())
    } else if vertex_count == 4 {
        let f = |idx| {
            let vertex_index = face_vertex_indices[idx] as usize;
            let vertices = &vertices[vertex_index];
            if vertices.len() != 3 {
                return Err(Triangulate3DError::ErrVertexIsNot3D { vertex_index });
            }

            Ok(glam::Vec3 {
                x: vertices[0],
                y: vertices[1],
                z: vertices[2],
            })
        };

        let vertices = [f(0)?, f(1)?, f(2)?, f(3)?];

        let d0 = (vertices[0] - vertices[2]).length_squared();
        let d1 = (vertices[1] - vertices[3]).length_squared();

        let (faces, new_edge) = if d1 < d0 {
            ([[0, 1, 3], [1, 2, 3]], [1, 3])
        } else {
            ([[0, 1, 2], [0, 2, 3]], [0, 2])
        };
        append_edge(new_edge);
        register_face(faces[0]);
        register_face(faces[1]);
        Ok(())
    } else {
        // As we're in 3D we need to take a careful approach for polygons over quads, as we need to create points
        // - either like in OS, we use earcut with a lot of management around
        // - same but with delaunator / other 2D CDT
        // - we implemenet DeWall (https://www.sciencedirect.com/science/article/abs/pii/S0010448597000821) / (https://github.com/OpenDelaunayVoronoi/DeWall-InCoDe)
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use std::assert_matches::assert_matches;

    #[test]
    fn test_triangulate_invalid_vertex_count() {
        let replace_face_partial = |_| panic!("should not be called");
        let append_edge = |_| panic!("should not be called");

        let indices = [0, 1];
        let vertices = [[0.4, 0.8, 1.0].as_slice(), [0.2, 1.2, 1.1].as_slice()];
        let res = super::triangulate3d(&indices, &vertices, replace_face_partial, append_edge);
        assert_eq!(
            res,
            Err(super::Triangulate3DError::ErrFaceIsNotAPolygon { vertex_count: 2 })
        )
    }

    #[test]
    fn test_triangulate_passthrough() {
        let mut replaced = None;
        let replace_face_partial = |triple| {
            if let Some(_) = replaced {
                panic!("set more than once")
            } else {
                replaced = Some(triple);
            }
        };
        let append_edge = |_| panic!("should not be called");

        let indices = [0, 1, 2];
        let vertices = [
            [0.4, 0.8, 1.0].as_slice(),
            [0.2, 1.2, 1.1].as_slice(),
            [0.9, 1.0, 1.0].as_slice(),
        ];
        let res = super::triangulate3d(&indices, &vertices, replace_face_partial, append_edge);
        assert_eq!(res, Ok(()));
        assert_eq!(replaced, Some([0, 1, 2]));
    }

    #[test]
    fn test_triangulate_quad() {
        let mut replaced = Vec::with_capacity(2);
        let replace_face_partial = |triple| replaced.push(triple);

        let mut appended = None;
        let append_edge = |new_edge| {
            if let Some(_) = appended {
                panic!("created moe than one edge")
            } else {
                appended = Some(new_edge);
            }
        };

        let indices = [0, 1, 2, 3];
        let vertices = [
            [0.0, 0.0, 0.0].as_slice(),
            [0.0, 1.0, 0.0].as_slice(),
            [1.0, 1.0, 0.0].as_slice(),
            [1.0, 0.0, 0.0].as_slice(),
        ];
        let res = super::triangulate3d(&indices, &vertices, replace_face_partial, append_edge);
        assert_eq!(res, Ok(()));
        assert_eq!(replaced.len(), 2);
        assert_matches!(appended, Some([0, 2]) | Some([1, 3]));
    }
}
