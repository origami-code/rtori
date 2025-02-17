//! Logic to import a FOLD frame into an Origami Simulator model solver
//! The import works in several phases which build on top of each other.
//! 1. `triangulation`: the FOLD frame's faces are triangulated, creating facet creases as needed.
//! 2. `transform`: the given FOLD frame's missing information is computed and kept as
//!     a `TransformedData` instance.
//! 3. `crease_geometry`: the `TransformedInput` is analysed for its creases, which are
//!     extracted as well, stored in `CreaseGeometry`. This is the data, combined with the fold frame input,
//!     that is sufficient to be loaded into an Origami Simulator model solver.
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

mod crease_geometry;
pub use crease_geometry::{preprocess, InputWithCreaseGeometry, PreprocessingError};

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
    PreprocessingError(crate::crease_geometry::PreprocessingError),
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

/// Load the preprocessed input into a given loader
pub fn import_preprocessed_in<'output, 'input, O, FI, PA, A>(
    output: &mut O,
    preprocessed: &'input InputWithCreaseGeometry<'input, FI, PA>,
    config: ImportConfig,
    allocator: A,
) -> Result<(), ImportError>
where
    O: rtori_os_model::LoaderDyn<'output> + 'output,
    FI: ImportInput,
    PA: core::alloc::Allocator,
    A: core::alloc::Allocator + Clone,
{
    preprocessed.load(output, config, allocator)
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
    let preprocessed = crate::crease_geometry::preprocess(input, allocator.clone())
        .map_err(|e| ImportError::PreprocessingError(e))?;
    let model_size = preprocessed.size();

    let mut output_base = output_factory(*model_size);
    {
        let output = &mut output_base;
        import_preprocessed_in(output, &preprocessed, config, allocator)?;
    }

    Ok(output_base)
}
