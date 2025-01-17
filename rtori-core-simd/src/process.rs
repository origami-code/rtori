use core::simd::{LaneCount, SupportedLaneCount};

use crate::{
    kernels::{self, d_per_node::PerNodeOutput},
    model::NodeFaceSpec,
};

use crate::simd_atoms::{SimdF32, SimdU32, SimdVec3F};

use crate::model::{
    CreaseFaceIndices, CreaseNeighbourhood, CreasesPhysicsLens, NodeBeamSpec, NodeGeometry,
};

/// Should be swapped
#[derive(Debug)]
#[repr(transparent)]
pub struct MemorableInput<'backer, const L: usize>
where
    LaneCount<L>: SupportedLaneCount,
{
    pub crease_fold_angle: &'backer mut [SimdF32<L>],
}

#[derive(Debug)]
pub struct ScratchInput<'backer, const L: usize>
where
    LaneCount<L>: SupportedLaneCount,
{
    pub crease_physics: &'backer mut [CreasesPhysicsLens<L>],
    pub face_normals: &'backer mut [SimdVec3F<L>],
    pub node_crease_forces: &'backer mut [SimdVec3F<L>],
    pub node_beam_forces: &'backer mut [SimdVec3F<L>],
    pub node_beam_error: &'backer mut [SimdF32<L>],
    pub node_face_forces: &'backer mut [SimdVec3F<L>],
    pub node_face_error: &'backer mut [SimdF32<L>],
}

#[derive(Debug)]
pub struct ReadOnlyInput<'backer, const L: usize>
where
    LaneCount<L>: SupportedLaneCount,
{
    /* Per-Node: RO Geometry (unchanging)*/
    pub node_geometry: &'backer [NodeGeometry<L>],

    /* Per-Node: RO Configs */
    pub node_positions_unchanging: &'backer [SimdVec3F<L>],
    pub node_external_forces: &'backer [SimdVec3F<L>],
    pub node_mass: &'backer [SimdF32<L>],
    pub node_fixed: &'backer [SimdU32<L>],

    /* Per-Node: R/W (double buffered) */
    pub node_position_offset: &'backer [SimdVec3F<L>],
    pub node_velocity: &'backer [SimdVec3F<L>],

    /* Per-Crease: RO Geometry (split as they are accessed separately) */
    pub crease_face_indices: &'backer [CreaseFaceIndices<L>],
    pub crease_neighbourhoods: &'backer [CreaseNeighbourhood<L>],

    /* Per-Crease: RO Config */
    pub crease_k: &'backer [SimdF32<L>],
    // pub crease_d: &'backer [SimdF32], // unused for now
    pub crease_target_fold_angle: &'backer [SimdF32<L>],

    /* Per-Crease: RO (per-iteration) (fold angles)*/
    pub crease_fold_angle: &'backer [SimdF32<L>],

    /* Per-Face: RO Geometry (split as they are used in different contexts) */
    pub face_indices: &'backer [[SimdU32<L>; 3]],
    pub face_nominal_angles: &'backer [SimdVec3F<L>],

    /* Per-Node-Crease: RO Geometry (todo: merge) */
    pub node_crease_crease_indices: &'backer [SimdU32<L>],
    pub node_crease_node_number: &'backer [SimdU32<L>],

    /* Per-Node-Beams: RO Geometry */
    pub node_beam_spec: &'backer [NodeBeamSpec<L>],

    /* Per-Node-Beams: RO Configs */
    pub node_beam_length: &'backer [SimdF32<L>],
    pub node_beam_k: &'backer [SimdF32<L>],
    pub node_beam_d: &'backer [SimdF32<L>],

    /* Per-Node-Face: RO */
    pub node_face_spec: &'backer [NodeFaceSpec<L>],

    pub crease_percentage: f32,
    pub dt: f32,
    pub face_stiffness: f32,
}

/// The parameter L should be the native vector size of the platform for highest efficiency
#[tracing::instrument]
pub fn process<'a, const L: usize>(
    input: &'a ReadOnlyInput<'a, L>,          // RO
    scratch: &'a mut ScratchInput<'a, L>,     // WO
    memorable: &'a mut MemorableInput<'a, L>, // WO
) -> impl ExactSizeIterator<Item = PerNodeOutput<L>> + use<'a, L>
where
    LaneCount<L>: SupportedLaneCount,
    simba::simd::Simd<core::simd::Simd<f32, L>>: nalgebra::SimdRealField,
{
    // This can be run in its own thread
    let (face_normals, fold_angles) = {
        // a: per-face
        let per_face_dest = &mut scratch.face_normals;
        {
            let per_face_input = kernels::a_per_face::PerFaceInputs {
                face_node_indices: &input.face_indices,
                node_positions_offset: &input.node_position_offset,
                node_positions_unchanging: &input.node_positions_unchanging,
            };

            let it = kernels::a_per_face::calculate_normals(&per_face_input);
            let dest = per_face_dest;
            assert!(it.len() <= dest.len());

            for (i, output) in it.enumerate() {
                dest[i] = output.face_normals;
            }
        }

        // b(a): per-crease fold-angles
        let per_crease_fold_angle_dest = &mut memorable.crease_fold_angle;
        {
            let per_crease_fold_angles_input =
                kernels::ba_per_crease_fold_angles::PerCreaseFoldAngleInput {
                    crease_face_indices: &input.crease_face_indices,
                    crease_neighbourhoods: &input.crease_neighbourhoods,
                    crease_fold_angle: &input.crease_fold_angle,

                    node_positions_offset: &input.node_position_offset,
                    node_positions_unchanging: &input.node_positions_unchanging,

                    face_normals: &scratch.face_normals,
                };

            let it = kernels::ba_per_crease_fold_angles::calculate_crease_fold_angles(
                &per_crease_fold_angles_input,
            );
            let dest = per_crease_fold_angle_dest;

            assert!(it.len() <= dest.len());

            for (i, output) in it.enumerate() {
                dest[i] = output;
            }
        }

        (&scratch.face_normals, &memorable.crease_fold_angle)
    };

    // This can also be run on its own thread
    let crease_physics = {
        let per_crease_physics_dest = &mut scratch.crease_physics;
        {
            let per_crease_physics_input = kernels::bb_per_crease_physics::PerCreasePhysicsInput {
                crease_neighbourhood: &input.crease_neighbourhoods,
                node_positions_unchanging: &input.node_positions_unchanging,
                node_positions_offset: &input.node_position_offset,
            };

            let it =
                kernels::bb_per_crease_physics::calculate_crease_physics(&per_crease_physics_input);
            let dest = per_crease_physics_dest;

            assert!(it.len() <= dest.len());

            for (i, output) in it.enumerate() {
                dest[i] = output;
            }
        }
        &scratch.crease_physics
    };

    // This needs the two previous ones to be over
    let per_node_crease_forces = {
        let per_node_crease_forces_dest = &mut scratch.node_crease_forces;
        {
            let per_node_crease_input = kernels::ca_per_node_crease::PerNodeCreaseInput {
                node_crease_indices: &input.node_crease_crease_indices,
                node_crease_node_number: &input.node_crease_node_number,
                crease_fold_angles: fold_angles,
                crease_physics: crease_physics,
                crease_k: &input.crease_k,
                crease_target_fold_angle: &input.crease_target_fold_angle,
                crease_face_indices: &input.crease_face_indices,
                face_indices: &input.face_indices,
                face_normals: face_normals,
                crease_percentage: input.crease_percentage,
            };

            let it =
                kernels::ca_per_node_crease::calculate_node_crease_forces(&per_node_crease_input);
            let dest = per_node_crease_forces_dest;
            assert!(it.len() <= dest.len());

            for (i, output) in it.enumerate() {
                dest[i] = output;
            }
        }
        &scratch.node_crease_forces
    };

    // Doesn't need anything to run except the previous position
    let (per_node_beam_forces, per_node_beam_error) = {
        let per_node_beam_forces_dest = &mut scratch.node_beam_forces;
        let per_node_beam_error_dest = &mut scratch.node_beam_error;
        {
            let per_node_beam_input = kernels::cb_per_node_beam::PerNodeBeamInput {
                beam_spec: &input.node_beam_spec,
                beam_k: &input.node_beam_k,
                beam_d: &input.node_beam_d,
                beam_length: &input.node_beam_length,
                node_positions_unchanging: &input.node_positions_unchanging,
                node_positions_offset: &input.node_position_offset,
                node_velocity: &input.node_velocity,
            };

            let it = kernels::cb_per_node_beam::calculate_node_beam_forces(&per_node_beam_input);

            let forces_dest = per_node_beam_forces_dest;
            assert!(it.len() <= forces_dest.len());

            let error_dest = per_node_beam_error_dest;
            assert!(it.len() <= error_dest.len());

            for (i, output) in it.enumerate() {
                forces_dest[i] = output.force;
                error_dest[i] = output.error;
            }
        }

        (&scratch.node_beam_forces, &scratch.node_beam_error)
    };

    let (per_node_face_forces, per_node_face_error) = {
        let per_node_face_forces_dest = &mut scratch.node_face_forces;
        let per_node_face_error_dest = &mut scratch.node_face_error;
        {
            let per_node_face_input = kernels::cc_per_node_face::PerNodeFaceInput {
                node_face_spec: &input.node_face_spec,
                node_positions_unchanging: &input.node_positions_unchanging,
                node_positions_offset: &input.node_position_offset,
                node_velocity: &input.node_velocity,
                face_node_indices: &input.face_indices,
                face_normals: &face_normals,
                face_nominal_angles: &input.face_nominal_angles,
                face_stiffness: input.face_stiffness,
            };

            let it = kernels::cc_per_node_face::calculate_node_face_forces(&per_node_face_input);

            let forces_dest = per_node_face_forces_dest;
            assert!(it.len() <= forces_dest.len());

            let error_dest = per_node_face_error_dest;
            assert!(it.len() <= error_dest.len());

            for (i, output) in it.enumerate() {
                forces_dest[i] = output.force;
                error_dest[i] = output.error;
            }
        }
        (&scratch.node_face_forces, &scratch.node_face_error)
    };

    {
        let per_node_input = kernels::d_per_node::PerNodeInput {
            node_positions_offset: &input.node_position_offset,
            node_velocity: &input.node_velocity,
            node_external_forces: &input.node_external_forces,
            node_mass: &input.node_mass,
            node_fixed: &input.node_fixed,
            node_crease_force: per_node_crease_forces,
            node_beam_force: per_node_beam_forces,
            node_face_force: per_node_face_forces,
            dt: input.dt,
        };
        kernels::d_per_node::calculate_node_position(per_node_input)
    }
}
