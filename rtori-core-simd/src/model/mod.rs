use core::{
    ops::{Deref, DerefMut},
    simd::{LaneCount, SupportedLaneCount},
};

use crate::simd_atoms::*;

mod aosoa;

mod crease_neighbourhood;
pub use crease_neighbourhood::*;

mod crease_physics;
pub use crease_physics::*;

mod node_geometry;
pub use node_geometry::*;

mod node_beam_spec;
pub use node_beam_spec::*;

mod node_crease_pointers;
pub use node_crease_pointers::*;

mod crease_face_indices;
pub use crease_face_indices::*;

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Eq, Ord)]
pub struct ModelSizes {
    pub node_count: usize,
    pub crease_count: usize,
    pub face_count: usize,
    pub node_crease_count: usize,
    pub node_beam_count: usize,
    pub node_face_count: usize,
}

/// Geometry Data: just a transparent passthrough
#[derive(Debug)]
#[repr(transparent)]
pub struct GeometryData<'backer, T>(pub &'backer [T]);

impl<T> GeometryData<'_, T> {
    pub const MEMORY_REQUIREMENTS: (usize, usize) =
        (Self::backing_size_unit(), Self::alignment_unit());

    /// In bytes
    pub const fn backing_size_unit() -> usize {
        core::mem::size_of::<T>()
    }

    /// In bytes
    pub const fn alignment_unit() -> usize {
        core::mem::align_of::<T>()
    }

    /// In bytes
    pub const fn backing_size_val(&self) -> usize {
        Self::backing_size_unit() * self.0.len()
    }
}

impl<'backer, T> Deref for GeometryData<'backer, T> {
    type Target = [T];

    fn deref(&self) -> &Self::Target {
        self.0
    }
}

/// Parameter Data: can be marked as dirty
#[derive(Debug)]
pub struct ParameterData<'backer, T> {
    pub data: &'backer [T],
    pub dirty: bool,
}

impl<T> ParameterData<'_, T> {
    pub const MEMORY_REQUIREMENTS: (usize, usize) =
        (Self::backing_size_unit(), Self::alignment_unit());

    /// In bytes
    pub const fn backing_size_unit() -> usize {
        core::mem::size_of::<T>()
    }

    /// In bytes
    pub const fn alignment_unit() -> usize {
        core::mem::align_of::<T>()
    }

    /// In bytes
    pub const fn backing_size_val(&self) -> usize {
        Self::backing_size_unit() * self.data.len()
    }
}

impl<'backer, T> Deref for ParameterData<'backer, T> {
    type Target = [T];

    fn deref(&self) -> &Self::Target {
        self.data
    }
}

/// Output Data: double buffered
#[derive(Debug)]
pub struct MemorableData<'backer, T> {
    pub front: &'backer mut [T],
    pub back: &'backer mut [T], // is mut required ? It might be required when letting the caller swap them
}

impl<T> MemorableData<'_, T> {
    pub const MEMORY_REQUIREMENTS: (usize, usize) =
        (Self::backing_size_unit(), Self::alignment_unit());

    /// In bytes
    pub const fn backing_size_unit() -> usize {
        core::mem::size_of::<T>() * 2
    }

    /// In bytes
    pub const fn alignment_unit() -> usize {
        core::mem::align_of::<T>()
    }

    /// In bytes
    pub const fn backing_size_val(&self) -> usize {
        assert!(self.front.len() == self.back.len());

        Self::backing_size_unit() * self.front.len()
    }
}

/// Scratch Data: single buffered
#[derive(Debug)]
#[repr(transparent)]
pub struct ScratchData<'backer, T>(pub &'backer mut [T]);

impl<T> ScratchData<'_, T> {
    pub const MEMORY_REQUIREMENTS: (usize, usize) =
        (Self::backing_size_unit(), Self::alignment_unit());

    /// In bytes
    pub const fn backing_size_unit() -> usize {
        core::mem::size_of::<T>()
    }

    /// In bytes
    pub const fn alignment_unit() -> usize {
        core::mem::align_of::<T>()
    }

    /// In bytes
    pub const fn backing_size_val(&self) -> usize {
        Self::backing_size_unit() * self.0.len()
    }
}

impl<'backer, T> Deref for ScratchData<'backer, T> {
    type Target = [T];

    fn deref(&self) -> &Self::Target {
        self.0
    }
}

impl<'backer, T> DerefMut for ScratchData<'backer, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.0
    }
}

///
/// Data layout:
/// - Always keep a group of different datasets as SoA if any of those cases is true:
///     - If they are not always accessed together (for geometry & parameters)
///     - If it is reasonable for them to be set or read separately from the outside (for parameters & R/W)
///
/// Glossary:
/// - SoA: Structure of Arrays
/// - AoSoA: Array of Structure of Arrays
///
#[derive(Debug)]
pub struct State<'backer, const L: usize>
where
    LaneCount<L>: SupportedLaneCount,
{
    /* Per-Node: RO Geometry (unchanging)*/
    pub node_geometry: GeometryData<'backer, NodeGeometry<L>>,

    /* Per-Node: RO Configs */
    pub node_positions_unchanging: ParameterData<'backer, SimdVec3F<L>>,
    pub node_external_forces: ParameterData<'backer, SimdVec3F<L>>,
    pub node_mass: ParameterData<'backer, SimdF32<L>>,
    pub node_fixed: ParameterData<'backer, SimdU32<L>>,

    /* Per-Node: R/W (double buffered) */
    pub node_position_offset: MemorableData<'backer, SimdVec3F<L>>,
    pub node_velocity: MemorableData<'backer, SimdVec3F<L>>,
    pub node_error: ScratchData<'backer, SimdF32<L>>,

    /* Per-Crease: RO Geometry (split as they are accessed separately) */
    pub crease_face_indices: GeometryData<'backer, CreaseFaceIndices<L>>,
    pub crease_neighbourhoods: GeometryData<'backer, CreaseNeighbourhood<L>>,

    /* Per-Crease: RO Config */
    pub crease_k: ParameterData<'backer, SimdF32<L>>,
    // pub crease_d: &'backer [SimdF32], // unused for now
    pub crease_target_fold_angle: ParameterData<'backer, SimdF32<L>>,

    /* Per-Crease: RW (fold angles)*/
    pub crease_fold_angle: MemorableData<'backer, SimdF32<L>>, // not scratch as we are its own consumers in the same pass (we have the read the previous ones)

    /* Per-Crease: RW (physics) (todo: merge into AoSoA for better locality)*/
    pub crease_physics: ScratchData<'backer, CreasesPhysicsLens<L>>,

    /* Per-Face: RO Geometry (split as they are used in different contexts) */
    pub face_indices: GeometryData<'backer, [SimdU32<L>; 3]>,
    pub face_nominal_angles: GeometryData<'backer, SimdVec3F<L>>,

    /* Per-Face: RW (normals) */
    pub face_normals: ScratchData<'backer, SimdVec3F<L>>,

    /* Per-Node-Crease: RO Geometry (todo: merge) */
    pub node_crease_crease_indices: GeometryData<'backer, SimdU32<L>>,
    pub node_crease_node_number: GeometryData<'backer, SimdU32<L>>,

    /* Per-Node-Crease: RW */
    pub node_crease_forces: ScratchData<'backer, SimdVec3F<L>>,

    /* Per-Node-Beams: RO Geometry (todo: merge) */
    pub node_beam_node_index: GeometryData<'backer, SimdU32<L>>,
    pub node_beam_length: GeometryData<'backer, SimdF32<L>>,
    pub node_beam_neighbour_index: GeometryData<'backer, SimdU32<L>>,

    /* Per-Node-Beams: RO Configs */
    pub node_beam_k: ParameterData<'backer, SimdF32<L>>,
    pub node_beam_d: ParameterData<'backer, SimdF32<L>>,

    /* Per-Node-Beams: RW */
    pub node_beam_forces: ScratchData<'backer, SimdVec3F<L>>,
    pub node_beam_error: ScratchData<'backer, SimdF32<L>>,

    /* Per-Node-Face: RO (todo: merge) */
    pub node_face_node_index: GeometryData<'backer, SimdU32<L>>,
    pub node_face_face_index: GeometryData<'backer, SimdU32<L>>,

    /* Per-Node-Face: RW */
    pub node_face_forces: ScratchData<'backer, SimdVec3F<L>>,
    pub node_face_error: ScratchData<'backer, SimdF32<L>>,

    pub crease_percentage: f32,
    pub dt: f32,
    pub face_stiffness: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DataConcept {
    PerNode,
    PerCrease,
    PerFace,
    PerNodeCrease,
    PerNodeBeam,
    PerNodeFace,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DataCharacteristic {
    pub concept: DataConcept,
    pub unit_size: usize,
    pub unit_alignment: usize,
}

pub const DATA_COUNT: usize = 31;

macro_rules! define (
    () => ([
        /* per_node */

        m!((0) PerNode(G) NodeGeometry<L>), /* node_geometry */
        m!((1) PerNode(P) SimdVec3F<L>), /* node_positions_unchanging */
        m!((2) PerNode(P) SimdVec3F<L>),/* node_external_forces */
        m!((3) PerNode(P) SimdF32<L>),/* node_mass */
        m!((4) PerNode(P) SimdU32<L>),/* node_fixed */
        m!((5) PerNode(M) SimdVec3F<L>),/* node_position_offset */
        m!((6) PerNode(M) SimdVec3F<L>),/* node_velocity */
        m!((7) PerNode(S) SimdF32<L>),/* node_error */

        /* per_crease */

        m!((8) PerCrease(G) CreaseFaceIndices<L>), /* crease_face_indices */
        m!((9) PerCrease(G) CreaseNeighbourhood<L>), /* crease_neighbourhoods */
        m!((10) PerCrease(P) SimdF32<L>), /* crease_k */
        m!((11) PerCrease(P) SimdF32<L>), /* crease_target_fold_angle */
        m!((12) PerCrease(M) SimdF32<L>), /* crease_fold_angle */
        m!((13) PerCrease(S) CreasesPhysicsLens<L>), /* creases_physics */

        /* per_face */

        m!((14) PerFace(G) [SimdU32<L>; 2]), /* face_indices */
        m!((15) PerFace(G) SimdVec3F<L>), /* face_nominal_angles */
        m!((16) PerFace(S) SimdVec3F<L>), /* face_normals */

        /* per_node_crease */

        m!((17) PerNodeCrease(G) SimdU32<L>), /* node_crease_crease_indices */
        m!((18) PerNodeCrease(G) SimdU32<L>), /* node_crease_node_number */
        m!((19) PerNodeCrease(S) SimdVec3F<L>), /* node_crease_forces */

        /* per_node_beam */

        m!((20) PerNodeBeam(G) SimdU32<L>), /* node_beam_node_index */
        m!((21) PerNodeBeam(G) SimdF32<L>), /* node_beam_length */
        m!((22) PerNodeBeam(G) SimdU32<L>), /* node_beam_neighbour_index */
        m!((23) PerNodeBeam(P) SimdF32<L>), /* node_beam_k */
        m!((24) PerNodeBeam(P) SimdF32<L>), /* node_beam_d */
        m!((25) PerNodeBeam(S) SimdVec3F<L>), /* node_beam_forces */
        m!((26) PerNodeBeam(S) SimdF32<L>), /* node_error */

        /* per_node_face */

        m!((27) PerNodeFace(G) SimdU32<L>), /* node_face_node_index */
        m!((28) PerNodeFace(G) SimdU32<L>), /* node_face_face_index */
        m!((29) PerNodeFace(S) SimdVec3F<L>), /* node_face_forces */
        m!((30) PerNodeFace(S) SimdF32<L>) /* node_face_error */
    ])
);

impl<'backer, const L: usize> State<'backer, L>
where
    LaneCount<L>: SupportedLaneCount,
{
    pub const DATA_CHARACTERISTICS: [DataCharacteristic; DATA_COUNT] =
        Self::compute_data_characteristics();

    /// The minimum size needed to host the required data
    pub const fn compute_data_characteristics() -> [DataCharacteristic; DATA_COUNT] {
        type G<T> = GeometryData<'static, T>;
        type P<T> = ParameterData<'static, T>;
        type S<T> = ScratchData<'static, T>;
        type M<T> = MemorableData<'static, T>;

        macro_rules! m(
            (($idx:expr) $concept:ident($class:ident) $field_type:ty) => (DataCharacteristic {
                concept: $concept,
                unit_size: ($class::<$field_type>::MEMORY_REQUIREMENTS).0,
                unit_alignment: ($class::<$field_type>::MEMORY_REQUIREMENTS).1,
            })
        );

        use DataConcept::*;
        let sizes_and_alignments = define!();

        sizes_and_alignments
    }

    pub const fn required_backing_size(model_size: &ModelSizes) -> usize {
        use DataConcept::*;

        let mut cursor = 0usize;
        let mut idx = 0usize;
        loop {
            if idx >= DATA_COUNT {
                break;
            }

            let characteristic = Self::DATA_CHARACTERISTICS[idx];

            let item_count = match characteristic.concept {
                PerNode => model_size.node_count,
                PerCrease => model_size.crease_count,
                PerFace => model_size.face_count,
                PerNodeCrease => model_size.node_crease_count,
                PerNodeBeam => model_size.node_beam_count,
                PerNodeFace => model_size.node_face_count,
            };

            let aligned = cursor.next_multiple_of(characteristic.unit_alignment);
            cursor = aligned + item_count * characteristic.unit_size;

            idx += 1;
        }

        cursor
    }
}
