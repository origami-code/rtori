mod dt;
pub use dt::*;

use crate::creases::Crease;
use core::alloc::Allocator;
use rtori_os_model::ModelSize;

pub enum InvalidationMask {
    None,
    InvalidateAll,
    // TODO: bitvec to do partial invalidation
}

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub(crate) struct CreaseNodePair {
    pub crease_index: u32,
    pub node_index: u32,
}

#[derive(Debug, Clone)]
pub(crate) struct PreprocessedData<A>
where
    A: Allocator,
{
    pub creases: alloc::vec::Vec<crate::creases::Crease, A>,

    /// Also called "inverted crease"
    pub node_creases_adjacent: alloc::vec::Vec<CreaseNodePair, A>,

    /// Also called just "crease" or "direct crease"
    pub node_creases_complement: alloc::vec::Vec<CreaseNodePair, A>,
}

impl<A> PreprocessedData<A>
where
    A: Allocator,
{
    const fn with_input<'input, I>(
        self,
        input: &'input I,
        size: ModelSize,
    ) -> PreprocessedInput<'input, I, A> {
        PreprocessedInput {
            input,
            size,
            preprocessed: self,
        }
    }
}

#[derive(Debug, Clone)]
pub struct PreprocessedInput<'input, I, A>
where
    A: Allocator,
{
    pub(crate) input: &'input I,
    pub(crate) size: rtori_os_model::ModelSize,
    pub(crate) preprocessed: PreprocessedData<A>,
}

impl<'input, I, A> PreprocessedInput<'input, I, A>
where
    A: Allocator,
{
    pub fn size(&self) -> &rtori_os_model::ModelSize {
        &self.size
    }
}

#[derive(Debug, Clone, Copy)]
pub enum PreprocessingError {
    ExtractCreasesError(crate::creases::ExtractCreasesIteratorError),
    EdgesVerticesInvalid { edge_index: u32 },
}

pub fn preprocess_data<'input, I, A>(
    input: &'input I,
    allocator: A,
) -> Result<PreprocessedData<A>, PreprocessingError>
where
    I: crate::creases::ExtractCreasesInput,
    A: Allocator + Clone,
{
    use crate::input::Proxy;

    let creases = {
        let creases_iter = crate::creases::extract_creases(input);
        let mut creases = alloc::vec::Vec::<crate::creases::Crease, _>::with_capacity_in(
            input.edges_vertices().count(),
            allocator.clone(),
        );
        for (i, crease_extraction_result) in creases_iter.enumerate() {
            let crease =
                crease_extraction_result.map_err(|e| PreprocessingError::ExtractCreasesError(e))?;
            creases.push(crease);
        }
        creases
    };

    let mut node_inv_creases = alloc::vec::Vec::<CreaseNodePair, _>::new_in(allocator.clone());
    let mut node_creases = alloc::vec::Vec::<CreaseNodePair, _>::new_in(allocator.clone());
    for (crease_index, crease) in creases.iter().enumerate() {
        let edge_index = crease.edge_index;

        // We'll need this at several points
        let vertex_indices = input
            .edges_vertices()
            .get(crease.edge_index as usize)
            .ok_or(PreprocessingError::EdgesVerticesInvalid {
                edge_index: crease.edge_index,
            })?;

        // First, fill in our (crease_index <-> node_index map for inverse creases)
        node_inv_creases.extend(vertex_indices.into_iter().map(|node_index| CreaseNodePair {
            crease_index: crease_index as u32,
            node_index,
        }));

        // Same but for direct creases
        node_creases.extend(crease.faces.into_iter().map(|face| CreaseNodePair {
            crease_index: crease_index as u32,
            node_index: face.complement_vertex_index,
        }));
    }

    Ok(PreprocessedData {
        creases,
        node_creases_adjacent: node_inv_creases,
        node_creases_complement: node_creases,
    })
}

fn compute_size<I>(input: &I, creases_count: u32, node_creases_count: u32) -> ModelSize
where
    I: crate::ImportInput,
{
    use crate::input::Proxy;

    let node_beams_count = input
        .vertices_edges()
        .iter()
        .fold(0, |acc, el| acc + el.len());
    let node_faces_count = input
        .vertices_faces()
        .iter()
        .fold(0, |acc, el| acc + el.len());

    rtori_os_model::ModelSize {
        nodes: input.vertices_coords().count() as u32,
        creases: creases_count,
        faces: input.faces_vertices().count() as u32,
        node_beams: node_beams_count as u32,
        node_creases: node_creases_count,
        node_faces: node_faces_count as u32,
    }
}

pub fn preprocess<'input, I, A>(
    input: &'input I,
    allocator: A,
) -> Result<PreprocessedInput<'input, I, A>, PreprocessingError>
where
    I: crate::input::ImportInput,
    A: Allocator + Clone,
{
    let preprocessed = preprocess_data(input, allocator)?;
    let size = compute_size(
        input,
        preprocessed.creases.len() as u32,
        (preprocessed.node_creases_adjacent.len() + preprocessed.node_creases_complement.len())
            as u32,
    );

    Ok(PreprocessedInput {
        input,
        size,
        preprocessed,
    })
}
