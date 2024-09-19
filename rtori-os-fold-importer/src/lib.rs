#![feature(impl_trait_in_assoc_type)]
#![feature(array_try_map)]
use creases::{
    extract_creases, ExtractCreasesError, ExtractCreasesIteratorError,
};
use fold_input::{FoldAssignment, Vector2U, Vector3F, Vector3U};
pub mod creases;
pub mod fold_input;
pub mod triangulation;
use fold_input::ImportInput;
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct NodeConfig {
    pub mass: f32,
    pub fixed: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct NodePointer {
    pub offset: u32,
    pub count: u16,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct NodeGeometry {
    pub crease: NodePointer,
    pub beam: NodePointer,
    pub face: NodePointer,
}

#[derive(Debug, Clone, Copy)]
pub struct CreaseGeometryFace {
    pub face_index: u32,
    pub complement_vertex_index: u32,
}

#[derive(Debug, Clone, Copy)]
pub struct CreaseGeometry {
    pub faces: [CreaseGeometryFace; 2],
}

#[derive(Debug, Clone, Copy)]
pub struct CreaseParameters {
    pub k: f32,
    pub d: f32,
    pub target_fold_angle: f32,
}

#[derive(Debug, Clone, Copy)]
pub struct NodeCreaseSpec {
    pub crease_index: u32,
    pub node_number: u32,
}

#[derive(Debug, Clone, Copy)]
pub struct NodeBeamSpec {
    pub node_index: u32,
    pub k: f32,
    pub d: f32,
    pub length: f32,
    pub neighbour_index: u32,
}

#[derive(Debug, Clone, Copy)]
pub struct NodeFaceSpec {
    pub node_index: u32,
    pub face_index: u32,
}

pub trait Output {
    fn set_node_position(&mut self, idx: usize, pos: [f32; 3]);
    fn set_node_external_forces(&mut self, idx: usize, pos: [f32; 3]);
    fn set_node_config(&mut self, idx: usize, config: NodeConfig);
    fn set_node_geometry(&mut self, idx: usize, geometry: NodeGeometry);

    fn set_crease_geometry(&mut self, idx: usize, geometry: CreaseGeometry);
    fn set_crease_parameters(&mut self, idx: usize, parameters: CreaseParameters);

    fn set_face_indices(&mut self, idx: usize, parameters: [u32; 3]);
    fn set_face_nominal_angles(&mut self, idx: usize, parameters: [f32; 3]);

    fn set_node_crease(&mut self, idx: usize, spec: NodeCreaseSpec);
    fn set_node_beam(&mut self, idx: usize, spec: NodeBeamSpec);
    fn set_node_face(&mut self, idx: usize, spec: NodeFaceSpec); // Unfinished
}

pub trait OutputFactory {
    type O: Output;

    fn create(
        &mut self,
        vertices_count: usize,
        crease_count: usize,
        faces_count: usize,
        node_beam_count: usize,
        node_creases_count: usize,
        node_faces_count: usize,
    ) -> Self::O;
}

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
        edge_index: u32
    },
    EdgesVerticesPointingToInvalidVertex {
        edge_index: u32,
        pointing_to: u32
    },
    CreaseExtractionError(ExtractCreasesIteratorError),
}

#[derive(Default, Clone, Copy)]
pub struct SetupConfig {
    pub default_axial_stiffness: f32,
    pub default_crease_stiffness: f32
}

fn setup<'a, O: Output, FI: ImportInput>(output: &mut O, input: &FI, config: SetupConfig) -> Result<(), ImportError> {
    for i in 0..input.vertices_coords().count() {
        output.set_node_config(
            i,
            NodeConfig {
                mass: 1.0,
                fixed: false,
            },
        );
    }

    for (face_index, face_vertices) in input.faces_vertices().iter().enumerate() {
        output.set_face_indices(face_index, face_vertices);

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

            output.set_face_nominal_angles(face_index, [x, y, z]);
        }
    }

    let creases = extract_creases(input);
    for (crease_index, res) in creases.enumerate() {
        let crease: creases::Crease = res.map_err(ImportError::CreaseExtractionError)?;

        let geometry = CreaseGeometry {
            faces: crease.faces.map(|f| CreaseGeometryFace {
                face_index: f.face_index,
                complement_vertex_index: f.complement_vertex_index,
            }),
        };
        output.set_crease_geometry(crease_index, geometry);

        let crease_stiffness = input
            .edges_crease_stiffnesses()
            .map_or(
                config.default_crease_stiffness,
                |v| v.get(crease.edge_index as usize)
                .expect("Crease refers to non-existing edge in edges_crease_stiffnesses")
            );

        let axial_stiffness = input
            .edges_axial_stiffnesses()
            .map_or(
                config.default_axial_stiffness,
                |v| v.get(crease.edge_index as usize)
                    .expect("Crease refers to non-existing edge in edges_axial_stiffnesses")
            );
            
        let vertices 
            = input
            .edges_vertices()
            .get(crease.edge_index as usize)
            .ok_or(ImportError::EdgesVerticesInvalid{
                edge_index: crease.edge_index
            })
            .and_then(|indices|
                indices.try_map(|index| 
                    input.vertices_coords()
                        .get(index as usize)
                        .ok_or(ImportError::EdgesVerticesPointingToInvalidVertex{
                            edge_index: crease.edge_index,
                            pointing_to: index
                        })
                )
            )?;

        let length = {
            let a = glam::Vec3::from(vertices[0]);
            let b = glam::Vec3::from(vertices[1]);

            (b - a).length()
        };

        let k = crease_stiffness * length;
        let d = axial_stiffness / length;

        let parameters = CreaseParameters {
            target_fold_angle: crease.fold_angle,
            k,
            d,
        };

        output.set_crease_parameters(crease_index, parameters);
    }

    let mut node_creases_offset = 0;
    let mut node_beams_offset = 0;
    let mut node_faces_offset = 0;
    for (node_index, node_faces) in input.vertices_faces() {
        
    }

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
