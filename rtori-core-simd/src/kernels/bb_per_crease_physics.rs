use core::ops::BitOr;
use core::simd::cmp::SimdPartialOrd;

use nalgebra::SimdComplexField;

use super::position;
use crate::model::CreaseGeometryLens;
use crate::{model::CreasesPhysicsLens, simd_atoms::*};

pub struct PerCreasePhysicsInput<'backer> {
    node_positions_unchanging: &'backer [SimdVec3F],
    node_positions_offset: &'backer [SimdVec3F],
    crease_geometry: &'backer [CreaseGeometryLens],
}

const TOL: f32 = 0.000001;

pub fn calculate_crease_physics<'a>(
    inputs: &'a PerCreasePhysicsInput<'a>,
) -> impl ExactSizeIterator<Item = CreasesPhysicsLens> + use<'a> {
    // First check
    let tol = simba::simd::Simd(SimdF32::splat(TOL));

    let invalid_value = SimdF32::splat(-1.0);

    inputs.crease_geometry.iter().map(move |geometry| {
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

        let crease_vector = node_eb - node_ea;
        let crease_length = crease_vector.norm();

        // First check: creases too small
        let too_short = crease_length.simd_abs().0.simd_lt(tol.0);

        // We can already abort if they are ALL invalid
        if too_short.all() {
            return CreasesPhysicsLens::invalid();
        }

        let crease_vector_normalized = crease_vector / crease_length;

        let vector_a = node_fa - node_ea;
        let vector_a_mag_sq = vector_a.magnitude_squared();
        let proj_a_length = crease_vector_normalized.dot(&vector_a);
        let dist_a = vector_a_mag_sq - proj_a_length * proj_a_length;
        let dist_a_too_small = dist_a.0.simd_le(tol.0);

        let vector_b = node_fb - node_ea; // not a typo ('ea')
        let vector_b_mag_sq = vector_b.magnitude_squared();
        let proj_b_length = crease_vector_normalized.dot(&vector_b);
        let dist_b = vector_b_mag_sq - proj_b_length * proj_b_length;
        let dist_b_too_small = dist_b.0.simd_le(tol.0);

        // Second check: distances too small
        let invalids = too_short.bitor(dist_a_too_small).bitor(dist_b_too_small);

        CreasesPhysicsLens {
            a_height: invalids.select(invalid_value, dist_a.0),
            a_coef: invalids.select(invalid_value, (proj_a_length / crease_length).0),
            b_height: invalids.select(invalid_value, dist_b.0),
            b_coef: invalids.select(invalid_value, (proj_b_length / crease_length).0),
        }
    })
}
