#![no_std]
#![cfg_attr(test, feature(assert_matches))]
#![feature(impl_trait_in_assoc_type)]
#![feature(array_try_map)]
#![feature(allocator_api)]
#![feature(btreemap_alloc)]
#![feature(map_try_insert)]
#![feature(iter_chain)]
use core::alloc::Allocator;

use creases::{extract_creases, ExtractCreasesIteratorError};
pub mod creases;
pub mod input;
pub mod triangulation;
use input::{ImportInput, Proxy};

extern crate alloc;
use alloc::vec::Vec;
use rtori_os_model::NodeBeamPointer;

#[cfg(feature = "fold")]
mod transform;

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

pub fn import<'output, O, FI, A>(
    output: &'output mut O,
    input: &FI,
    config: ImportConfig,
    allocator: A,
) -> Result<(), ImportError>
where
    O: rtori_os_model::LoaderDyn<'output>,
    FI: ImportInput,
    A: core::alloc::Allocator + Clone,
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

    let mut node_inv_creases = alloc::vec::Vec::new_in(allocator.clone());
    let mut node_creases = alloc::vec::Vec::new_in(allocator.clone());

    let creases = extract_creases(input);
    for (crease_index, res) in creases.enumerate() {
        let crease: creases::Crease = res.map_err(ImportError::CreaseExtractionError)?;

        // We'll need this at several points
        let vertex_indices = input
            .edges_vertices()
            .get(crease.edge_index as usize)
            .ok_or(ImportError::EdgesVerticesInvalid {
                edge_index: crease.edge_index,
            })?;

        // First, fill in our (crease_index <-> node_index map for inverse creases)
        vertex_indices.into_iter().for_each(|vertex_index| {
            node_inv_creases.push((crease_index, vertex_index));
        });

        // Same but for direct creases
        crease.faces.iter().for_each(|face| {
            node_creases.push((crease_index, face.complement_vertex_index));
        });

        let geometry = rtori_os_model::CreaseGeometry {
            faces: crease.faces.map(|f| rtori_os_model::CreaseGeometryFace {
                face_index: f.face_index,
                complement_vertex_index: f.complement_vertex_index,
            }),
        };
        output.copy_crease_geometry(&[geometry], crease_index as u32);

        let crease_stiffness = input
            .edges_crease_stiffnesses()
            .and_then(|v| {
                v.get(crease.edge_index as usize)
                    .expect("Crease refers to non-existing edge in edges_crease_stiffnesses")
            })
            .unwrap_or(config.default_crease_stiffness);

        let axial_stiffness = input
            .edges_axial_stiffnesses()
            .and_then(|v| {
                v.get(crease.edge_index as usize)
                    .expect("Crease refers to non-existing edge in edges_axial_stiffnesses")
            })
            .unwrap_or(config.default_axial_stiffness);

        let vertices = vertex_indices.try_map(|index| {
            input.vertices_coords().get(index as usize).ok_or(
                ImportError::EdgesVerticesPointingToInvalidVertex {
                    edge_index: crease.edge_index,
                    pointing_to: index,
                },
            )
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

    let vertices_count = input.vertices_coords().count();

    fn create_node_index_to_crease_index<S, A, U, V>(
        source: S,
        vertices_count: usize,
        allocator: A,
    ) -> Vec<Vec<U, A>, A>
    where
        S: AsRef<[(U, V)]>,
        A: Allocator + Clone,
        U: Copy,
        V: Copy,
        usize: TryFrom<V>,
        <usize as TryFrom<V>>::Error: core::fmt::Debug,
    {
        let mut mapping = alloc::vec::Vec::with_capacity_in(vertices_count, allocator.clone());
        mapping.resize(vertices_count, alloc::vec::Vec::new_in(allocator.clone()));

        source.as_ref().iter().for_each(|(to, from)| {
            mapping[usize::try_from(*from).unwrap()].push(*to);
        });

        mapping
    }

    let node_index_to_crease_indices =
        create_node_index_to_crease_index(node_creases, vertices_count, allocator.clone());
    assert_eq!(node_index_to_crease_indices.len(), vertices_count);

    let node_index_to_inv_crease_indices =
        create_node_index_to_crease_index(node_inv_creases, vertices_count, allocator.clone());
    assert_eq!(node_index_to_inv_crease_indices.len(), vertices_count);

    let mut node_creases_offset = 0;
    
    let mut node_beams_offset = 0;
    let mut node_faces_offset = 0;
    for (i, (node_creases_for_this_node, node_inv_creases_for_this_node, vertices_faces, vertices_edges)) in itertools::izip!(
        node_index_to_crease_indices,
        node_index_to_inv_crease_indices,
        input.vertices_faces().iter(),
        input.vertices_edges().iter()
    )
    .enumerate()
    {   
        let crease_count = node_creases_for_this_node.len() + node_inv_creases_for_this_node.len();

        let geometry = rtori_os_model::NodeGeometry {
            crease: rtori_os_model::NodeCreasePointer { offset: node_creases_offset, count: crease_count },
            beam: NodeBeamPointer {offset: node_beams_offset, count: vertices_edges.len()},
            face: NodeBeamPointer {offset: node_faces_offset, count: vertices_faces.len()},
        }; 
        output.copy_node_geometry(from, offset);
    }

    //for (node_index, node_faces) in input.vertices_faces() {}

    unimplemented!()
}
