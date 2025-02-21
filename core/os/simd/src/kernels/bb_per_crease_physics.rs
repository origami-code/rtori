use core::ops::BitOr;
use core::simd::cmp::SimdPartialOrd;
use core::simd::{LaneCount, SupportedLaneCount};

use nalgebra::{RealField, SimdComplexField};

use simd_common::*;

use super::{operations, position};
use crate::model::CreaseNeighbourhood;
use crate::model::CreasesPhysicsLens;

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

            // The node naming is the following:
            //
            // face #
            //       0  ||  1
            // diagram:
            //          ea
            //         /||\
            //        / || \
            //       /  ||  \
            //      fa••||••fb
            //       \  ||  /
            //        \ || /
            //         \||/
            //          eb
            //
            // names (original paper naming in []):
            // - fa [p1]: complementary node (from face #0)
            // - fb [p2]: complementary node (from face #1)
            // - ea [p3]: adjacent node (vertex 0 from edge)
            // - eb [p4]: adjacent node (vertex 1 from edge)
            //
            // It follows that |eb-ea| is the crease vector

            let node_fa = get_position(neighbourhood.complement_node_indices[0]);
            let node_fb = get_position(neighbourhood.complement_node_indices[1]);

            let node_ea = get_position(neighbourhood.adjacent_node_indices[0]);
            let node_eb = get_position(neighbourhood.adjacent_node_indices[1]);

            tracing::event!(
                tracing::Level::TRACE,
                "Node Indices In Crease:\n\t
                fa: {:?}
                fb: {:?}
                ea: {:?}
                eb: {:?}
            ",
                neighbourhood.complement_node_indices[0],
                neighbourhood.complement_node_indices[1],
                neighbourhood.adjacent_node_indices[0],
                neighbourhood.adjacent_node_indices[1]
            );

            let crease_vector = node_eb - node_ea;
            let crease_length = crease_vector.norm();

            // First check: creases too small
            let too_short = crease_length.simd_abs().0.simd_lt(tol.0);

            // We can already abort if they are ALL invalid
            if too_short.all() {
                return CreasesPhysicsLens::invalid();
            }

            let crease_vector_normalized = crease_vector / crease_length;

            // Calculates the projection of one of the two complementary nodes
            let calculate_projection = |complementary_node: nalgebra::Vector3<
                simba::simd::Simd<core::simd::Simd<f32, { L }>>,
            >| {
                let vector = complementary_node - node_ea; // not a typo 'ea'
                let vector_mag_sq = vector.magnitude_squared();
                let proj_length = crease_vector_normalized.dot(&vector);

                let dist = {
                    use core::simd::num::SimdFloat;
                    //use core::simd::StdFloat;

                    // sqrt(abs(v.x^2 + v.y^2 + v.z^2 - proj^2))
                    simba::simd::Simd((vector_mag_sq - proj_length * proj_length).0.abs())
                        .simd_sqrt()
                        .0
                };

                /* 2025-01-15 */
                /*println!("
                bb_per_crease_physics:
                    ea: {node_ea:?}
                    eb: {node_eb:?}
                    fa: {node_fa:?}
                    fb: {node_fb:?}
                    complementary_node: {complementary_node:?}
                    crease_vector: {crease_vector:?}
                    crease_vector_normalized: {crease_vector_normalized:?}
                    on vector: {vector:?}
                    proj_length: {proj_length:?} (dot product)
                    dist: {dist:?}");*/

                let dist_too_small = dist.simd_le(tol.0);

                (proj_length, dist, dist_too_small)
            };

            let (proj_a_length, dist_a, dist_a_too_small) = calculate_projection(node_fa);
            let (proj_b_length, dist_b, dist_b_too_small) = calculate_projection(node_fb);

            // Second check: distances too small
            let invalids = too_short.bitor(dist_a_too_small).bitor(dist_b_too_small);

            let g =
                |dist: core::simd::Simd<f32, { L }>,
                 proj_length: simba::simd::Simd<core::simd::Simd<f32, { L }>>| {
                    (
                        invalids.select(invalid_value, dist),
                        invalids.select(invalid_value, (proj_length / crease_length).0),
                    )
                };

            let (a_height, a_coef) = g(dist_a, proj_a_length);
            let (b_height, b_coef) = g(dist_b, proj_b_length);
            /* 2025-01-15 */
            tracing::event!(
                tracing::Level::TRACE,
                "bb_per_crease_physics:
                dist_a_too_small?: {dist_a_too_small:?}
                dist_a: {dist_a:?}
                proj_a_length: {proj_a_length:?}
                a_height: {a_height:?}
                a_coef: {a_coef:?}
            "
            );
            tracing::event!(
                tracing::Level::TRACE,
                "bb_per_crease_physics:
                dist_b_too_small?: {dist_b_too_small:?}
                dist_b: {dist_b:?}
                proj_b_length: {proj_b_length:?}
                b_height: {b_height:?}
                b_coef: {b_coef:?}
            "
            );
            let res = CreasesPhysicsLens {
                a_height,
                a_coef,
                b_height,
                b_coef,
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
