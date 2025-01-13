use core::ops::BitOr;
use core::simd::cmp::SimdPartialOrd;
use core::simd::{LaneCount, SupportedLaneCount};

use nalgebra::{RealField, SimdComplexField};

use super::{operations, position};
use crate::model::CreaseNeighbourhood;
use crate::{model::CreasesPhysicsLens, simd_atoms::*};

#[derive(Debug)]
pub struct PerCreasePhysicsInput<'backer, const L: usize>
where
    LaneCount<L>: SupportedLaneCount,
{
    pub crease_neighbourhood: &'backer [CreaseNeighbourhood<L>],
    pub node_positions_unchanging: &'backer [SimdVec3F<L>],
    pub node_positions_offset: &'backer [SimdVec3F<L>],
}

const TOL: f32 = 0.000001;

pub fn calculate_crease_physics<'a, const L: usize>(
    inputs: &'a PerCreasePhysicsInput<'a, L>,
) -> impl ExactSizeIterator<Item = CreasesPhysicsLens<L>> + use<'a, L>
where
    LaneCount<L>: SupportedLaneCount,
    simba::simd::Simd<core::simd::Simd<f32, L>>: nalgebra::SimdRealField,
{
    // First check
    let tol = simba::simd::Simd(SimdF32::splat(TOL));

    let invalid_value = SimdF32::splat(-1.0);

    inputs
        .crease_neighbourhood
        .iter()
        .map(move |neighbourhood| {
            let get_position = #[inline(always)]
            |indices: SimdU32<L>| {
                position::get_positions_for_indices(
                    &inputs.node_positions_unchanging,
                    &inputs.node_positions_offset,
                    indices,
                )
            };

            // /* 2025-01-13 */ println!("node_positions_offset: {:?}", &inputs.node_positions_offset);

            let node_fa = get_position(neighbourhood.complement_node_indices[0]);
            let node_fb = get_position(neighbourhood.complement_node_indices[1]);

            let node_ea = get_position(neighbourhood.adjacent_node_indices[0]);
            let node_eb = get_position(neighbourhood.adjacent_node_indices[1]);

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

            let res = CreasesPhysicsLens {
                a_height: invalids.select(invalid_value, dist_a.0),
                a_coef: invalids.select(invalid_value, (proj_a_length / crease_length).0),
                b_height: invalids.select(invalid_value, dist_b.0),
                b_coef: invalids.select(invalid_value, (proj_b_length / crease_length).0),
            };
            operations::debug::check_nans_simd_msg(
                res.a_height,
                "bb_per_crease_physics",
                "a_height",
            );
            operations::debug::check_nans_simd_msg(res.a_coef, "bb_per_crease_physics", "a_coef");
            operations::debug::check_nans_simd_msg(
                res.b_height,
                "bb_per_crease_physics",
                "b_height",
            );
            operations::debug::check_nans_simd_msg(res.b_coef, "bb_per_crease_physics", "b_coef");

            res
        })
}
