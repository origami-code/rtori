#![cfg_attr(test, feature(assert_matches))]
#![feature(impl_trait_in_assoc_type)]
#![feature(array_try_map)]
use creases::{extract_creases, ExtractCreasesError, ExtractCreasesIteratorError};
use fold_input::{FoldAssignment, Vector2U, Vector3F, Vector3U};
pub mod creases;
pub mod fold_input;
pub mod triangulation;
use fold_input::ImportInput;

use crate::fold_input::Proxy;

#[derive(Debug, Clone, Copy)]
pub enum ImportError {
    IncorrectFaceIndices {
        face_index: u32,
        vertex_number: u8,
        points_to_vertex: u32,
        vertex_count: u32,
    },
    EdgesVerticesInvalid {
        edge_index: u32,
    },
    EdgesVerticesPointingToInvalidVertex {
        edge_index: u32,
        pointing_to: u32,
    },
    CreaseExtractionError(ExtractCreasesIteratorError),
}

#[derive(Default, Clone, Copy)]
pub struct ImportConfig {
    pub default_axial_stiffness: f32,
    pub default_crease_stiffness: f32,
}

pub fn import<'output, O, FI>(
    output: &'output mut O,
    input: &FI,
    config: ImportConfig,
) -> Result<(), ImportError>
where
    O: rtori_os_model::LoaderDyn<'output>,
    FI: ImportInput,
{
    for i in 0..input.vertices_coords().count() {
        const BASE: rtori_os_model::NodeConfig = rtori_os_model::NodeConfig {
            mass: 1.0,
            fixed: 0,
            _reserved: [0; 3],
        };
        output.copy_node_config(&[BASE], i as u32);
    }

    for (face_index, face_vertices) in input.faces_vertices().iter().enumerate() {
        output.copy_face_indices(
            &[rtori_os_model::Vector3U(face_vertices)],
            face_index as u32,
        );

        {
            let pos_for_node = |vertex_number: u8| {
                let node_index = face_vertices[usize::from(vertex_number)];

                input.vertices_coords().get(node_index as usize).ok_or(
                    ImportError::IncorrectFaceIndices {
                        face_index: face_index as u32,
                        vertex_number,
                        points_to_vertex: node_index,
                        vertex_count: input.vertices_coords().count() as u32,
                    },
                )
            };

            let [a, b, c] = [0, 1, 2].try_map(pos_for_node)?;
            let ab = glam::Vec3::from(b) - glam::Vec3::from(a);
            let ac = glam::Vec3::from(c) - glam::Vec3::from(a);
            let bc = glam::Vec3::from(c) - glam::Vec3::from(b);

            let x = f32::acos(glam::Vec3::dot(ab, ac));
            let y = f32::acos(-1f32 * glam::Vec3::dot(ab, bc));
            let z = f32::acos(glam::Vec3::dot(ac, bc));

            output.copy_face_nominal_angles(
                &[rtori_os_model::Vector3F([x, y, z])],
                face_index as u32,
            );
        }
    }

    let creases = extract_creases(input);
    for (crease_index, res) in creases.enumerate() {
        let crease: creases::Crease = res.map_err(ImportError::CreaseExtractionError)?;

        let geometry = rtori_os_model::CreaseGeometry {
            faces: crease.faces.map(|f| rtori_os_model::CreaseGeometryFace {
                face_index: f.face_index,
                complement_vertex_index: f.complement_vertex_index,
            }),
        };
        output.copy_crease_geometry(&[geometry], crease_index as u32);

        let crease_stiffness =
            input
                .edges_crease_stiffnesses()
                .map_or(config.default_crease_stiffness, |v| {
                    v.get(crease.edge_index as usize)
                        .expect("Crease refers to non-existing edge in edges_crease_stiffnesses")
                });

        let axial_stiffness =
            input
                .edges_axial_stiffnesses()
                .map_or(config.default_axial_stiffness, |v| {
                    v.get(crease.edge_index as usize)
                        .expect("Crease refers to non-existing edge in edges_axial_stiffnesses")
                });

        let vertices = input
            .edges_vertices()
            .get(crease.edge_index as usize)
            .ok_or(ImportError::EdgesVerticesInvalid {
                edge_index: crease.edge_index,
            })
            .and_then(|indices| {
                indices.try_map(|index| {
                    input.vertices_coords().get(index as usize).ok_or(
                        ImportError::EdgesVerticesPointingToInvalidVertex {
                            edge_index: crease.edge_index,
                            pointing_to: index,
                        },
                    )
                })
            })?;

        let length = {
            let a = glam::Vec3::from(vertices[0]);
            let b = glam::Vec3::from(vertices[1]);

            (b - a).length()
        };

        let k = crease_stiffness * length;
        let d = axial_stiffness / length;

        let parameters = rtori_os_model::CreaseParameters {
            target_fold_angle: crease.fold_angle,
            k,
            d,
        };

        output.copy_crease_parameters(&[parameters], crease_index as u32);
    }

    let mut node_creases_offset = 0;
    let mut node_beams_offset = 0;
    let mut node_faces_offset = 0;
    //for (node_index, node_faces) in input.vertices_faces() {}

    unimplemented!()
}

/*
pub fn import<'a, O: OutputFactory, FI: FoldInput>(
    target: &mut O,
    fi: &'a FI
) -> Result<(), ()> {
    // 1. Triangulate
    // 2. Extract creases
    let output
}*/
