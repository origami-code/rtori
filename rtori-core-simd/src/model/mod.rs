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

mod node_face_spec;
pub use node_face_spec::*;

mod node_crease_pointers;
pub use node_crease_pointers::*;

mod crease_face_indices;
pub use crease_face_indices::*;

/// Geometry Data: just a transparent passthrough
#[derive(Debug, Default)]
#[repr(transparent)]
pub struct GeometryData<'backer, T>(pub &'backer mut [T]);

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

    pub const fn empty() -> Self {
        Self(&mut [])
    }
}

impl<'backer, T> Deref for GeometryData<'backer, T> {
    type Target = [T];

    fn deref(&self) -> &Self::Target {
        self.0
    }
}

impl<'backer, T> DerefMut for GeometryData<'backer, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.0
    }
}

/// Parameter Data: can be marked as dirty
#[derive(Debug, Default)]
pub struct ParameterData<'backer, T> {
    pub data: &'backer mut [T],
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

    pub const fn empty() -> Self {
        Self {
            data: &mut [],
            dirty: false,
        }
    }
}

impl<'backer, T> Deref for ParameterData<'backer, T> {
    type Target = [T];

    fn deref(&self) -> &Self::Target {
        self.data
    }
}

impl<'backer, T> DerefMut for ParameterData<'backer, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.data
    }
}

/// Output Data: double buffered
#[derive(Debug, Default)]
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

    pub const fn swap(&mut self) {
        core::mem::swap(&mut self.front, &mut self.back)
    }

    pub const fn empty() -> Self {
        Self {
            front: &mut [],
            back: &mut [],
        }
    }
}

/// Scratch Data: single buffered
#[derive(Debug, Default)]
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

    pub const fn empty() -> Self {
        Self(&mut [])
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
    pub node_beam_spec: GeometryData<'backer, NodeBeamSpec<L>>,

    /* Per-Node-Beams: RO Configs */
    pub node_beam_length: ParameterData<'backer, SimdF32<L>>,
    pub node_beam_k: ParameterData<'backer, SimdF32<L>>,
    pub node_beam_d: ParameterData<'backer, SimdF32<L>>,

    /* Per-Node-Beams: RW */
    pub node_beam_forces: ScratchData<'backer, SimdVec3F<L>>,
    pub node_beam_error: ScratchData<'backer, SimdF32<L>>,

    /* Per-Node-Face: RO (todo: merge) */
    pub node_face_spec: GeometryData<'backer, NodeFaceSpec<L>>,

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

pub const DATA_COUNT: usize = 29;

macro_rules! define_inner(
    /* per_node */
    (0) => {m!((0) node_geometry PerNode(G) NodeGeometry<L>)};
    (1) => {m!((1) node_positions_unchanging PerNode(P) SimdVec3F<L>)};
    (2) => {m!((2) node_external_forces PerNode(P) SimdVec3F<L>)};
    (3) => {m!((3) node_mass PerNode(P) SimdF32<L>)};
    (4) => {m!((4) node_fixed PerNode(P) SimdU32<L>)};
    (5) => {m!((5) node_position_offset PerNode(M) SimdVec3F<L>)};
    (6) => {m!((6) node_velocity PerNode(M) SimdVec3F<L>)};
    (7) => {m!((7) node_error PerNode(S) SimdF32<L>)};

    /* per_crease */
    (8) => {m!((8) crease_face_indices PerCrease(G) CreaseFaceIndices<L>)};
    (9) => {m!((9) crease_neighbourhoods PerCrease(G) CreaseNeighbourhood<L>)};
    (10) => {m!((10) crease_k PerCrease(P) SimdF32<L>)};
    (11) => {m!((12) crease_target_fold_angle PerCrease(P) SimdF32<L>)};
    (12) => {m!((12) crease_fold_angle PerCrease(M) SimdF32<L>)};
    (13) => {m!((13) crease_physics PerCrease(S) CreasesPhysicsLens<L>)};

    /* per_face */
    (14) => {m!((14) face_indices PerFace(G) [SimdU32<L>; 3])};
    (15) => {m!((15) face_nominal_angles PerFace(G) SimdVec3F<L>)};
    (16) => {m!((16) face_normals PerFace(S) SimdVec3F<L>)};

    /* per_node_crease */
    (17) => {m!((17) node_crease_crease_indices PerNodeCrease(G) SimdU32<L>)};
    (18) => {m!((18) node_crease_node_number PerNodeCrease(G) SimdU32<L>)};
    (19) => {m!((19) node_crease_forces PerNodeCrease(S) SimdVec3F<L>)};

    /* per_node_beam */
    (20) => {m!((20) node_beam_spec PerNodeBeam(G) NodeBeamSpec<L>)};
    (21) => {m!((21) node_beam_length PerNodeBeam(P) SimdF32<L>)};
    (22) => {m!((22) node_beam_k PerNodeBeam(P) SimdF32<L>)};
    (23) => {m!((23) node_beam_d PerNodeBeam(P) SimdF32<L>)};
    (24) => {m!((24) node_beam_forces PerNodeBeam(S) SimdVec3F<L>)};
    (25) => {m!((25) node_beam_error PerNodeBeam(S) SimdF32<L>)};

    /* per_node_face */
    (26) => {m!((26) node_face_spec PerNodeFace(G) NodeFaceSpec<L>)};
    (27) => {m!((27) node_face_forces PerNodeFace(S) SimdVec3F<L>)};
    (28) => {m!((28) node_face_error PerNodeFace(S) SimdF32<L>)}
);

macro_rules! define (
    () => ([
        define_inner!(0),
        define_inner!(1),
        define_inner!(2),
        define_inner!(3),
        define_inner!(4),
        define_inner!(5),
        define_inner!(6),
        define_inner!(7),
        define_inner!(8),
        define_inner!(9),
        define_inner!(10),
        define_inner!(11),
        define_inner!(12),
        define_inner!(13),
        define_inner!(14),
        define_inner!(15),
        define_inner!(16),
        define_inner!(17),
        define_inner!(18),
        define_inner!(19),
        define_inner!(20),
        define_inner!(21),
        define_inner!(22),
        define_inner!(23),
        define_inner!(24),
        define_inner!(25),
        define_inner!(26),
        define_inner!(27),
        define_inner!(28)
    ])
);

impl<'backer, const L: usize> State<'backer, L>
where
    LaneCount<L>: SupportedLaneCount,
{
    pub const fn size(&self) -> rtori_os_model::ModelSize {
        rtori_os_model::ModelSize {
            nodes: (self.node_positions_unchanging.data.len() * L) as u32,
            creases: (self.crease_target_fold_angle.data.len() * L) as u32,
            faces: (self.face_indices.0.len() * L) as u32,
            node_beams: (self.node_beam_length.data.len() * L) as u32,
            node_creases: (self.node_crease_crease_indices.0.len() * L) as u32,
            node_faces: (self.node_face_spec.0.len() * L) as u32,
        }
    }

    pub const DATA_CHARACTERISTICS: [DataCharacteristic; DATA_COUNT] =
        Self::compute_data_characteristics();

    /// The minimum size needed to host the required data
    const fn compute_data_characteristics() -> [DataCharacteristic; DATA_COUNT] {
        type G<T> = GeometryData<'static, T>;
        type P<T> = ParameterData<'static, T>;
        type S<T> = ScratchData<'static, T>;
        type M<T> = MemorableData<'static, T>;

        // Todo: incorporate assumption to check types
        macro_rules! m(
            (($idx:expr) $field_name:ident $concept:ident($class:ident) $field_type:ty) => (DataCharacteristic {
                concept: $concept,
                unit_size: ($class::<$field_type>::MEMORY_REQUIREMENTS).0,
                unit_alignment: ($class::<$field_type>::MEMORY_REQUIREMENTS).1,
            })
        );

        use DataConcept::*;
        let sizes_and_alignments = define!();

        sizes_and_alignments
    }

    const fn empty() -> Self {
        Self {
            node_geometry: GeometryData::empty(),
            node_positions_unchanging: ParameterData::empty(),
            node_external_forces: ParameterData::empty(),
            node_mass: ParameterData::empty(),
            node_fixed: ParameterData::empty(),
            node_position_offset: MemorableData::empty(),
            node_velocity: MemorableData::empty(),
            node_error: ScratchData::empty(),
            crease_face_indices: GeometryData::empty(),
            crease_neighbourhoods: GeometryData::empty(),
            crease_k: ParameterData::empty(),
            crease_target_fold_angle: ParameterData::empty(),
            crease_fold_angle: MemorableData::empty(),
            crease_physics: ScratchData::empty(),
            face_indices: GeometryData::empty(),
            face_nominal_angles: GeometryData::empty(),
            face_normals: ScratchData::empty(),
            node_crease_crease_indices: GeometryData::empty(),
            node_crease_node_number: GeometryData::empty(),
            node_crease_forces: ScratchData::empty(),
            node_beam_spec: GeometryData::empty(),
            node_beam_length: ParameterData::empty(),
            node_beam_k: ParameterData::empty(),
            node_beam_d: ParameterData::empty(),
            node_beam_forces: ScratchData::empty(),
            node_beam_error: ScratchData::empty(),
            node_face_spec: GeometryData::empty(),
            node_face_forces: ScratchData::empty(),
            node_face_error: ScratchData::empty(),
            crease_percentage: 0.66,
            dt: 0.001,
            face_stiffness: 1.0,
        }
    }

    pub fn from_slice(
        model_size: &rtori_os_model::ModelSize,
        slice: &'backer mut [u8],
    ) -> Result<(Self, &'backer mut [u8]), ()> {
        use DataConcept::*;

        let mut output = Self::empty();

        let mut rest = slice;

        for idx in 0..DATA_COUNT {
            let characteristic = Self::DATA_CHARACTERISTICS[idx];

            let lane_count = match characteristic.concept {
                PerNode => model_size.nodes,
                PerCrease => model_size.creases,
                PerFace => model_size.faces,
                PerNodeCrease => model_size.node_creases,
                PerNodeBeam => model_size.node_beams,
                PerNodeFace => model_size.node_faces,
            } as usize;
            let item_count = lane_count.div_ceil(L);

            // First, find at what point do we split between the first and 2nd slice
            let ptr = rest.as_ptr();
            let offset_bytes = ptr.align_offset(characteristic.unit_alignment);
            if offset_bytes > rest.len() {
                return Err(());
            }

            // Then, we split at this point already, that will be dead space
            let (_dead_space, mid_and_post) = rest.split_at_mut(offset_bytes);
            let required_size_in_bytes = item_count * characteristic.unit_size;
            let (mid, post) = mid_and_post.split_at_mut(required_size_in_bytes);
            assert_eq!(mid.len(), required_size_in_bytes);

            rest = post;

            #[derive(Debug, Clone, Copy)]
            enum TransmuteSpecialError {
                InputSliceIsNotAligned {
                    alignment: usize,
                    /// Do not dereference
                    address_as_ptr: *const u8,
                },
                InputSliceSizeNotAMultiple {
                    element_size_in_bytes: usize,
                    input_size_in_bytes: usize,
                },
            }

            impl core::fmt::Display for TransmuteSpecialError {
                fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                    match self {
                        Self::InputSliceIsNotAligned { alignment, address_as_ptr } => write!(f, "Input slice is not aligned on the expected boundary, pointer is {:?}, alignment requirement is {}B (modulo {}B)", address_as_ptr, alignment, (*address_as_ptr as usize) % alignment),
                        Self::InputSliceSizeNotAMultiple { element_size_in_bytes, input_size_in_bytes } =>write!(f, "Error: the slice's length ({}B) should be a multiple of the target type's size ({}B), but it isn't (modulo is {}B)", input_size_in_bytes, element_size_in_bytes, input_size_in_bytes % element_size_in_bytes)
                    }
                }
            }

            #[inline]
            unsafe fn transmute_special<'a, T>(
                slice: &'a mut [u8],
            ) -> Result<&'a mut [T], TransmuteSpecialError> {
                if (slice.as_ptr() as usize) % core::mem::align_of::<T>() != 0 {
                    return Err(TransmuteSpecialError::InputSliceIsNotAligned {
                        alignment: core::mem::align_of::<T>(),
                        address_as_ptr: slice.as_ptr(),
                    });
                }

                if slice.len() % core::mem::size_of::<T>() != 0 {
                    return Err(TransmuteSpecialError::InputSliceSizeNotAMultiple {
                        element_size_in_bytes: core::mem::size_of::<T>(),
                        input_size_in_bytes: slice.len(),
                    });
                }

                let (pre, target, post) = slice.align_to_mut::<T>();
                assert_eq!(pre.len(), 0, "one of the expectations is that we have already offsetted using ptr.align_offset");
                assert_eq!(
                    post.len(),
                    0,
                    "while transmutting a slice which should be aligned and of the right size"
                );

                Ok(target)
            }

            unsafe fn create_memorable<'a, T>(
                slice: &'a mut [u8],
                unit_count: usize,
            ) -> Result<MemorableData<'a, T>, TransmuteSpecialError> {
                let transmuted = transmute_special::<T>(slice)?;

                let (front, back) = transmuted.split_at_mut(unit_count);
                assert!(front.len() == unit_count && back.len() == unit_count);

                Ok(MemorableData { front, back })
            }

            unsafe fn create_geometry<'a, T>(
                slice: &'a mut [u8],
                unit_count: usize,
            ) -> Result<GeometryData<'a, T>, TransmuteSpecialError> {
                let transmuted = transmute_special::<T>(slice)?;
                assert_eq!(
                    transmuted.len(),
                    unit_count,
                    "transmute_special should return the expected unit count (of {})",
                    unit_count
                );

                Ok(GeometryData(transmuted))
            }

            unsafe fn create_parameter<'a, T>(
                slice: &'a mut [u8],
                unit_count: usize,
            ) -> Result<ParameterData<'a, T>, TransmuteSpecialError> {
                let transmuted = transmute_special::<T>(slice)?;
                assert_eq!(transmuted.len(), unit_count);
                Ok(ParameterData {
                    data: transmuted,
                    dirty: true,
                })
            }

            unsafe fn create_scratch<'a, T>(
                slice: &'a mut [u8],
                unit_count: usize,
            ) -> Result<ScratchData<'a, T>, TransmuteSpecialError> {
                let transmuted = transmute_special::<T>(slice)?;
                assert_eq!(transmuted.len(), unit_count);
                Ok(ScratchData(transmuted))
            }

            macro_rules! inner_message(
                (($idx:expr) $field_name:ident $concept:ident($class:ident) $field_type:ty) => (
                    concat!(stringify!($idx), " => ", stringify!($field_name), " [ type ", stringify!($field_type), "/ concept ", stringify!($concept), "] ")
                );
            );

            macro_rules! rhs(
                (@with $create_func:ident ($idx:expr) $field_name:ident $concept:ident($class:ident) $field_type:ty) => (
                    unsafe {$create_func(mid, item_count)}.expect(inner_message!(($idx) $field_name $concept(M) $field_type))
                );
            );

            macro_rules! arm(
                (@with $create_func:ident ($idx:expr) $field_name:ident $concept:ident($class:ident) $field_type:ty) => (
                    output.$field_name = rhs!(@with $create_func ($idx) $field_name $concept($class) $field_type)
                );
            );

            macro_rules! m(
                (($idx:expr) $field_name:ident $concept:ident(M) $field_type:ty) => (
                    arm!(@with create_memorable ($idx) $field_name $concept(M) $field_type)
                );
                (($idx:expr) $field_name:ident $concept:ident(G) $field_type:ty) => (
                    arm!(@with create_geometry ($idx) $field_name $concept(G) $field_type)
                );
                (($idx:expr) $field_name:ident $concept:ident(P) $field_type:ty) => (
                    arm!(@with create_parameter ($idx) $field_name $concept(P) $field_type)
                );
                (($idx:expr) $field_name:ident $concept:ident(S) $field_type:ty) => (
                    arm!(@with create_scratch ($idx) $field_name $concept(S) $field_type)
                );
            );

            // We know that we're aligned due to our align_offset call, so we're good to go
            match idx {
                0 => define_inner!(0),
                1 => define_inner!(1),
                2 => define_inner!(2),
                3 => define_inner!(3),
                4 => define_inner!(4),
                5 => define_inner!(5),
                6 => define_inner!(6),
                7 => define_inner!(7),
                8 => define_inner!(8),
                9 => define_inner!(9),
                10 => define_inner!(10),
                11 => define_inner!(11),
                12 => define_inner!(12),
                13 => define_inner!(13),
                14 => define_inner!(14),
                15 => define_inner!(15),
                16 => define_inner!(16),
                17 => define_inner!(17),
                18 => define_inner!(18),
                19 => define_inner!(19),
                20 => define_inner!(20),
                21 => define_inner!(21),
                22 => define_inner!(22),
                23 => define_inner!(23),
                24 => define_inner!(24),
                25 => define_inner!(25),
                26 => define_inner!(26),
                27 => define_inner!(27),
                28 => define_inner!(28),
                DATA_COUNT.. => unreachable!(),
            }
        }

        Ok((output, rest))
    }

    pub const fn required_backing_size(model_size: &rtori_os_model::ModelSize) -> usize {
        use DataConcept::*;

        let mut cursor = 0usize;
        let mut idx = 0usize;
        loop {
            if idx >= DATA_COUNT {
                break;
            }

            let characteristic = Self::DATA_CHARACTERISTICS[idx];

            let item_count = match characteristic.concept {
                PerNode => model_size.nodes,
                PerCrease => model_size.creases,
                PerFace => model_size.faces,
                PerNodeCrease => model_size.node_creases,
                PerNodeBeam => model_size.node_beams,
                PerNodeFace => model_size.node_faces,
            } as usize;

            let aligned = cursor.next_multiple_of(characteristic.unit_alignment);
            cursor = aligned + item_count * characteristic.unit_size;

            idx += 1;
        }

        cursor
    }
}
