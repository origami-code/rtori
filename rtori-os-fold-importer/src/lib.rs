#![no_std]
#![cfg_attr(test, feature(assert_matches))]
#![feature(impl_trait_in_assoc_type)]
#![feature(array_try_map)]
#![feature(allocator_api)]
#![feature(btreemap_alloc)]
#![feature(map_try_insert)]
#![feature(iter_chain)]
use core::alloc::Allocator;

pub mod creases;
pub mod input;
pub mod triangulation;
use input::{ImportInput, Proxy};

extern crate alloc;
use alloc::vec::Vec;
use rtori_os_model::{NodeBeamSpec, NodeCreaseSpec};

mod preprocess;
pub use preprocess::{preprocess, PreprocessedInput, PreprocessingError};

#[cfg(any(test, feature = "fold"))]
pub mod transform;

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
    PreprocessingError(crate::preprocess::PreprocessingError),
}

#[derive(Clone, Copy)]
pub struct ImportConfig {
    pub default_axial_stiffness: f32,
    pub default_crease_stiffness: f32,
    pub default_mass: f32,
    pub damping_percentage: f32,
}

impl ImportConfig {
    pub const DEFAULT: Self = Self {
        default_axial_stiffness: 20.0,
        default_crease_stiffness: 0.7,
        default_mass: 1.0,
        damping_percentage: 0.45,
    };
}

impl Default for ImportConfig {
    fn default() -> Self {
        Self::DEFAULT
    }
}

pub fn import<'output, 'input, F, O, FI>(
    output_factory: F,
    input: &'input FI,
    config: ImportConfig,
) -> Result<O, ImportError>
where
    F: FnOnce(rtori_os_model::ModelSize) -> O,
    O: rtori_os_model::LoaderDyn<'output> + 'output,
    FI: ImportInput,
{
    import_in(output_factory, input, config, alloc::alloc::Global)
}

pub fn import_preprocessed_in<'output, 'input, O, FI, PA, A>(
    output: &mut O,
    preprocessed: &'input PreprocessedInput<'input, FI, PA>,
    config: ImportConfig,
    allocator: A,
) -> Result<(), ImportError>
where
    O: rtori_os_model::LoaderDyn<'output> + 'output,
    FI: ImportInput,
    PA: core::alloc::Allocator,
    A: core::alloc::Allocator + Clone,
{
    let input = preprocessed.input;
    let creases = &preprocessed.preprocessed.creases;
    let node_inv_creases = &preprocessed.preprocessed.node_creases_adjacent;
    let node_creases = &preprocessed.preprocessed.node_creases_complement;

    for (i, vertex) in input.vertices_coords().iter().enumerate() {
        output.copy_node_position(&[rtori_os_model::Vector3F(vertex)], i as u32);

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
            let ab = (glam::Vec3::from(b) - glam::Vec3::from(a)).normalize();
            let ac = (glam::Vec3::from(c) - glam::Vec3::from(a)).normalize();
            let bc = (glam::Vec3::from(c) - glam::Vec3::from(b)).normalize();

            let x = f32::acos(glam::Vec3::dot(ab, ac));
            let y = f32::acos(-1f32 * glam::Vec3::dot(ab, bc));
            let z = f32::acos(glam::Vec3::dot(ac, bc));

            output.copy_face_nominal_angles(
                &[rtori_os_model::Vector3F([x, y, z])],
                face_index as u32,
            );
        }
    }

    for (crease_index, crease) in creases.iter().enumerate() {
        // We'll need this at several points
        let vertex_indices = input
            .edges_vertices()
            .get(crease.edge_index as usize)
            .ok_or(ImportError::EdgesVerticesInvalid {
                edge_index: crease.edge_index,
            })?;

        let geometry = rtori_os_model::CreaseGeometry {
            face_indices: crease.faces.map(|f| f.face_index),
            complementary_node_indices: crease.faces.map(|f| f.complement_vertex_index),
            adjacent_node_indices: vertex_indices,
        };
        output.copy_crease_geometry(&[geometry], crease_index as u32);

        let crease_stiffness = input
            .edges_crease_stiffnesses()
            .and_then(|v| {
                v.get(crease.edge_index as usize)
                    .expect("Crease refers to non-existing edge in edges_crease_stiffnesses")
            })
            .unwrap_or(config.default_crease_stiffness);

        /*let axial_stiffness = input
        .edges_axial_stiffnesses()
        .and_then(|v| {
            v.get(crease.edge_index as usize)
                .expect("Crease refers to non-existing edge in edges_axial_stiffnesses")
        })
        .unwrap_or(config.default_axial_stiffness);*/

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
        let d = 0.0f32; //axial_stiffness / length;

        let parameters = rtori_os_model::CreaseParameters {
            target_fold_angle: crease.fold_angle,
            k,
            d,
        };

        output.copy_crease_parameters(&[parameters], crease_index as u32);
    }

    let vertices_count = input.vertices_coords().count();

    fn create_node_index_to_crease_index<S, A>(
        source: S,
        vertices_count: usize,
        allocator: A,
    ) -> Vec<Vec<crate::creases::CreaseIndex, A>, A>
    where
        S: AsRef<[crate::preprocess::CreaseNodePair]>,
        A: Allocator + Clone,
    {
        let mut mapping = alloc::vec::Vec::with_capacity_in(vertices_count, allocator.clone());
        mapping.resize(vertices_count, alloc::vec::Vec::new_in(allocator.clone()));

        source.as_ref().iter().for_each(|pair| {
            mapping[usize::try_from(pair.node_index).unwrap()].push(pair.crease_index);
        });

        mapping
    }

    let node_index_to_crease_indices =
        create_node_index_to_crease_index(node_creases, vertices_count, allocator.clone());
    assert_eq!(node_index_to_crease_indices.len(), vertices_count);

    let node_index_to_inv_crease_indices =
        create_node_index_to_crease_index(node_inv_creases, vertices_count, allocator.clone());
    assert_eq!(node_index_to_inv_crease_indices.len(), vertices_count);

    let mut node_creases_cursor = 0;
    let mut node_beams_cursor = 0;
    let mut node_faces_cursor = 0;
    for (
        i,
        (
            node_creases_for_this_node,
            node_inv_creases_for_this_node,
            vertices_faces,
            vertices_edges,
        ),
    ) in itertools::izip!(
        node_index_to_crease_indices,
        node_index_to_inv_crease_indices,
        input.vertices_faces().iter(),
        input.vertices_edges().iter()
    )
    .enumerate()
    {
        let node_index = i as u32;

        let crease_count =
            (node_creases_for_this_node.len() + node_inv_creases_for_this_node.len()) as u32;
        let beams_count = vertices_edges.len() as u32;
        let faces_count = vertices_faces.len() as u32;

        let geometry = rtori_os_model::NodeGeometry {
            crease: rtori_os_model::NodeCreasePointer {
                offset: node_creases_cursor,
                count: crease_count,
            },
            beam: rtori_os_model::NodeBeamPointer {
                offset: node_beams_cursor,
                count: beams_count,
            },
            face: rtori_os_model::NodeFacePointer {
                offset: node_faces_cursor,
                count: faces_count,
            },
        };
        output.copy_node_geometry(&[geometry], node_index);

        // Load node-crease
        {
            fn process_node_crease<'input, 'loader, L, EV>(
                output: &mut L,
                node_index: u32,
                local_node_crease_index: u32,
                crease_index: u32,
                crease: &creases::Crease,
                edges_vertices: &EV,
            ) where
                L: rtori_os_model::LoaderDyn<'loader>,
                EV: Proxy<'input, Output = [u32; 2]>,
            {
                #[derive(Copy, Debug, Clone, PartialEq)]
                #[repr(u8)]
                enum NodeNumber {
                    N1 = 1,
                    N2 = 2,
                    N3 = 3,
                    N4 = 4,
                }

                fn get_node_number_from_index<'a, EV>(
                    edges_vertices: &EV,
                    crease: &creases::Crease,
                    node_index: u32,
                ) -> NodeNumber
                where
                    EV: Proxy<'a, Output = [u32; 2]>,
                {
                    if node_index == crease.faces[0].complement_vertex_index {
                        NodeNumber::N1
                    } else if node_index == crease.faces[1].complement_vertex_index {
                        NodeNumber::N2
                    } else {
                        let edge_vertex_indices = edges_vertices
                            .get(crease.edge_index as usize)
                            .expect("edge should be defined");
                        if node_index == edge_vertex_indices[0] {
                            NodeNumber::N3
                        } else if node_index == edge_vertex_indices[1] {
                            NodeNumber::N4
                        } else {
                            panic!("Not in crease")
                        }
                    }
                }

                let node_number_for_crease = get_node_number_from_index(
                    // ...
                    edges_vertices,
                    &crease,
                    node_index,
                );

                output.copy_node_crease(
                    &[NodeCreaseSpec {
                        crease_index: crease_index as u32,
                        node_number: node_number_for_crease as u8 as u32,
                    }],
                    local_node_crease_index as u32,
                );
            }

            let edges_vertices = input.edges_vertices();
            let node_creases_len = node_creases_for_this_node.len();
            node_creases_for_this_node
                .into_iter()
                .map(|crease_index| (crease_index, creases[crease_index as usize]))
                .enumerate()
                .for_each(|(local_node_crease_index, (crease_index, crease))| {
                    process_node_crease(
                        output,
                        node_index,
                        node_creases_cursor + local_node_crease_index as u32,
                        crease_index as u32,
                        &crease,
                        &edges_vertices,
                    );
                });
            node_creases_cursor += node_creases_len as u32;

            let node_inv_creases_len = node_inv_creases_for_this_node.len();
            node_inv_creases_for_this_node
                .into_iter()
                .map(|crease_index| (crease_index, creases[crease_index as usize]))
                .enumerate()
                .for_each(|(local_node_crease_index, (crease_index, crease))| {
                    process_node_crease(
                        output,
                        node_index,
                        node_creases_cursor + local_node_crease_index as u32,
                        crease_index as u32,
                        &crease,
                        &edges_vertices,
                    );
                });
            node_creases_cursor += node_inv_creases_len as u32;
        }
        // Load node-beams
        {
            let edge_vertex_indices: <FI as ImportInput>::EdgeVertices<'_> = input.edges_vertices();
            vertices_edges
                .iter()
                .map(|edge_index| {
                    (
                        edge_index,
                        edge_vertex_indices.get(*edge_index as usize).unwrap(),
                    )
                })
                .enumerate()
                .for_each(
                    |(node_beam_local_index, (edge_index, edge_vertex_indices))| {
                        let edge_vertices_coords: [[f32; 3]; 2] = edge_vertex_indices
                            .map(|index| input.vertices_coords().get(index as usize).unwrap());

                        let length = {
                            let a = glam::Vec3::from(edge_vertices_coords[0]);
                            let b = glam::Vec3::from(edge_vertices_coords[1]);

                            (b - a).length()
                        };

                        let default_mass = config.default_mass;
                        // TODO: recover the mass of the vertices
                        let min_mass = default_mass;
                        let axial_stiffness = input
                            .edges_axial_stiffnesses()
                            .and_then(|proxy| proxy.get(*edge_index as usize))
                            .flatten()
                            .unwrap_or(config.default_axial_stiffness);

                        let k = axial_stiffness / length;
                        let d = config.damping_percentage * 2.0 * f32::sqrt(k * min_mass);
                        let this_node_index_in_edge = match edge_vertex_indices {
                            [a, _] if a == node_index => 0,
                            [_, a] if a == node_index => 1,
                            _ => unreachable!(),
                        };
                        let neighbour_index_in_edge = 1 - this_node_index_in_edge;
                        let neighbour_index = edge_vertex_indices[neighbour_index_in_edge];

                        let spec = NodeBeamSpec {
                            node_index,
                            k,
                            d,
                            length,
                            neighbour_index,
                        };

                        output.copy_node_beam(
                            &[spec],
                            node_beams_cursor + (node_beam_local_index as u32),
                        )
                    },
                );
            node_beams_cursor += beams_count;
        }

        // Load node-faces
        vertices_faces.iter().enumerate().for_each(
            |(node_faces_offset_local, node_face_face_index)| {
                let index = node_faces_cursor + (node_faces_offset_local as u32);
                output.copy_node_face(
                    &[rtori_os_model::NodeFaceSpec {
                        node_index,
                        face_index: *node_face_face_index,
                    }],
                    index,
                );
            },
        );
        node_faces_cursor += faces_count;
    }

    //for (node_index, node_faces) in input.vertices_faces() {}

    Ok(())
}

pub fn import_in<'output, 'input, F, O, FI, A>(
    output_factory: F,
    input: &'input FI,
    config: ImportConfig,
    allocator: A,
) -> Result<O, ImportError>
where
    F: FnOnce(rtori_os_model::ModelSize) -> O,
    O: rtori_os_model::LoaderDyn<'output> + 'output,
    FI: ImportInput,
    A: core::alloc::Allocator + Clone,
{
    let preprocessed = crate::preprocess::preprocess(input, allocator.clone())
        .map_err(|e| ImportError::PreprocessingError(e))?;
    let model_size = preprocessed.size();

    let mut output_base = output_factory(*model_size);
    {
        let output = &mut output_base;
        import_preprocessed_in(output, &preprocessed, config, allocator)?;
    }

    Ok(output_base)
}
