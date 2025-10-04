//! Logic to import a FOLD frame into an Origami Simulator model solver
//! The import works in several phases which build on top of each other.
//!
//! 1. `triangulation`
//!     The FOLD frame's faces are triangulated, creating facet creases as needed.
//!     Mapping between the new faces and the previous ones is generated.
//! 2. `supplement`
//!     The given FOLD frame's missing information is computed and kept as
//!     a `FoldSupplement` instance. When combined with the input, this is a `SupplementedInput`,
//!     which implements the `ImportInput` trait needed to import.
//! 3. `crease_geometry`
//!     The `SupplementedInput` is analysed for its creases, which are
//!     extracted as well, stored in `CreaseGeometry`. This is the data, combined with the fold frame input,
//!     that is sufficient to be loaded into an Origami Simulator model solver.
#![no_std]
#![cfg_attr(test, feature(assert_matches))]
#![feature(impl_trait_in_assoc_type)]
#![feature(array_try_map)]
#![feature(allocator_api)]
#![feature(btreemap_alloc)]
#![feature(map_try_insert)]
use core::alloc::Allocator;

pub mod creases;
pub mod input;
pub mod triangulation;
use input::{ImportInput, Proxy};

extern crate alloc;
use alloc::vec::Vec;
use rtori_os_model::{NodeBeamSpec, NodeCreaseSpec};

mod crease_geometry;
pub use crease_geometry::{InputWithCreaseGeometry, PreprocessingError};

#[cfg(any(test, feature = "fold"))]
pub mod supplement;

#[derive(Debug, Clone, Copy)]
pub enum ImportError {
    /// In `faces_vertices`, face {face_index}'s vertex #{vertex_number} refers to vertex index {points_to_vertex}, but only {vertex_count} vertices are defined
    IncorrectFaceIndices {
        face_index: u32,
        vertex_number: u8,
        points_to_vertex: u32,
        vertex_count: u32,
    },
    /// Crease {crease_index} refers to an edge {edge_index} that is not present in the input
    InvalidCrease {
        crease_index: u32,
        edge_index: u32,
    },
    /// In `edges_vertices`, {edge_index}'s member #{edge_member} refers to vertex index {pointing_to}, but only {vertex_count} vertices are defined
    EdgesVerticesPointingToInvalidVertex {
        edge_index: u32,
        edge_member: u8,
        pointing_to: u32,
        vertex_count: u32,
    },
    PreprocessingError(crate::crease_geometry::PreprocessingError),
}

impl core::fmt::Display for ImportError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::IncorrectFaceIndices { face_index, vertex_number, points_to_vertex, vertex_count }
                => write!(f, "in `faces_vertices`, face {face_index}'s vertex #{vertex_number} refers to vertex index {points_to_vertex}, but only {vertex_count} vertices are defined"),
            Self::InvalidCrease { crease_index, edge_index }
                => write!(f, "crease {crease_index} refers to an edge {edge_index} that is not present in the input"),
            Self::EdgesVerticesPointingToInvalidVertex { edge_index, edge_member, pointing_to, vertex_count }
                => write!(f, "in `edges_vertices`, {edge_index}'s member #{edge_member} refers to vertex index {pointing_to}, but only {vertex_count} vertices are defined"),
            Self::PreprocessingError(p) => write!(f, "preprocessing error: {p}")
        }
    }
}
impl core::error::Error for ImportError {}

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
    let with_crease_geometry =
        crate::crease_geometry::InputWithCreaseGeometry::process(input, allocator.clone())
            .map_err(|e| ImportError::PreprocessingError(e))?;

    let model_size = with_crease_geometry.compute_size();
    let mut output_base = output_factory(model_size);
    {
        let output = &mut output_base;
        with_crease_geometry.load(output, config, allocator)?;
    }

    Ok(output_base)
}
