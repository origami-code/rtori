use super::position;
use crate::model::CreaseGeometry;
use crate::{model::CreasesPhysics, simd_atoms::*};

pub struct PerCreasePhysicsInput<'backer> {
    node_positions_unchanging: &'backer [SimdVec3F],
    node_positions_offset: &'backer [SimdVec3F],
    crease_geometry: &'backer [CreaseGeometry],
}

const TOL: f32 = 0.000001;

pub fn calculate_crease_physics<'a>(
    inputs: &'a PerCreasePhysicsInput<'a>,
) -> impl ExactSizeIterator<Item = CreasesPhysics> + use<'a> {
    inputs.crease_geometry.iter().map(|geometry| {
        let get_position = #[inline(always)]
        |indices: SimdU32| {
            position::get_positions_for_indices(
                &inputs.node_positions_unchanging,
                &inputs.node_positions_offset,
                indices,
            )
        };

        let node_fa = get_position(geometry.complement_node_indices[0]);
        let node_fb = get_position(geometry.complement_node_indices[1]);

        let node_ea = get_position(geometry.adjacent_node_indices[0]);
        let node_eb = get_position(geometry.adjacent_node_indices[1]);

        // First check
        /*

        let tol = simba::simd::Simd(
            SimdF32::splat(TOL)
        );
         */
        let crease_vector = node_eb - node_ea;
        let crease_length = crease_vector.norm();

        let crease_vector_normalized = crease_vector / crease_length;

        let vector_a = node_fa - node_ea;
        let vector_a_mag_sq = vector_a.magnitude_squared();
        let proj_a_length = crease_vector_normalized.dot(&vector_a);
        let dist_a = vector_a_mag_sq - proj_a_length * proj_a_length;

        let vector_b = node_fb - node_ea; // not a type ('ea')
        let vector_b_mag_sq = vector_b.magnitude_squared();
        let proj_b_length = crease_vector_normalized.dot(&vector_b);
        let dist_b = vector_b_mag_sq - proj_b_length * proj_b_length;

        // SEcond check

        CreasesPhysics {
            a_height: dist_a.0,
            a_coef: (proj_a_length / crease_length).0,
            b_height: dist_b.0,
            b_coef: (proj_b_length / crease_length).0,
        }
    })
}
