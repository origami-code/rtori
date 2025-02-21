use core::simd::{LaneCount, SupportedLaneCount};

use simd_common::{convert_nalgebra::to_nalgebra3, *};

use super::{gather::gather_vec3f_1, position};
use crate::model::{CreaseFaceIndices, CreaseNeighbourhood};
use nalgebra::{SimdComplexField, SimdRealField};
use nalgebra::{SimdPartialOrd, SimdValue};

#[derive(Debug)]
pub struct PerCreaseFoldAngleInput<'backer, const L: usize>
where
    LaneCount<L>: SupportedLaneCount,
{
    pub crease_face_indices: &'backer [CreaseFaceIndices<L>],
    pub crease_neighbourhoods: &'backer [CreaseNeighbourhood<L>],
    pub crease_fold_angle: &'backer [SimdF32<L>],

    pub node_positions_offset: &'backer [SimdVec3F<L>],
    pub node_positions_unchanging: &'backer [SimdVec3F<L>],

    pub face_normals: &'backer [SimdVec3F<L>],
}

pub struct PerCreaseFoldAngleOutput<'backer, const L: usize>
where
    LaneCount<L>: SupportedLaneCount,
{
    crease_fold_angle: &'backer mut [SimdF32<L>],
}

#[tracing::instrument]
pub fn calculate_crease_fold_angles<'a, const L: usize>(
    inputs: &'a PerCreaseFoldAngleInput<'a, L>,
) -> impl ExactSizeIterator<Item = SimdF32<L>> + use<'a, L>
where
    LaneCount<L>: SupportedLaneCount,
    simba::simd::Simd<core::simd::Simd<f32, L>>:
        num_traits::Num + num_traits::NumAssign + SimdComplexField + SimdRealField,
{
    itertools::izip!(
        inputs.crease_face_indices,
        inputs.crease_neighbourhoods,
        inputs.crease_fold_angle
    )
    .map(|(face_indices, neighbourhoods, previous_fold_angles)| {
        let g = #[inline(always)]
        |face_index| {
            let face_indices_a = face_indices.0[face_index];
            let normals_a = gather_vec3f_1(&inputs.face_normals, face_indices_a);
            let normals_a = to_nalgebra3(normals_a);
            normals_a
        };

        let normals_a = g(0);
        let normals_b = g(1);

        let normals_dot = {
            let normals_dot_unclamped = normals_a.dot(&normals_b);
            tracing::event!(tracing::Level::TRACE, "normals_dot_unclamped: {normals_dot_unclamped:?}\n\tnormals_a {:?}: {normals_a:?}\n\tnormals_b {:?}: {normals_b:?}", face_indices.0[0], face_indices.0[1]);

            simba::simd::Simd::simd_clamp(
                normals_dot_unclamped,
                simba::simd::Simd(SimdF32::splat(-1.0)),
                simba::simd::Simd(SimdF32::splat(1.0)),
            )
        };

        let get_adjacent = |face_index| {
            position::get_positions_for_indices(
                inputs.node_positions_unchanging,
                inputs.node_positions_offset,
                neighbourhoods.adjacent_node_indices[face_index],
            )
        };

        let vertex_a = get_adjacent(0);
        let vertex_b = get_adjacent(1);

        let ab = vertex_b - vertex_a;
        tracing::event!(tracing::Level::TRACE, "\n\tvertex_a (node indices: {:?}) {vertex_a:?}\n\tvertex_b (node indices: {:?}) {vertex_b:?}\n\tab {ab:?}\n\tneighbourhoods {neighbourhoods:?}\n\tnormals_dot: {normals_dot:?}", neighbourhoods.adjacent_node_indices[0], neighbourhoods.adjacent_node_indices[1]);
        let crease_vector = ab.normalize();

        let x = normals_dot;
        let y = (normals_a.cross(&crease_vector)).dot(&normals_b);

        // OPTIMIZATION(aab 2025-02-16):
        //
        // We bring in SLEEF here (or its reimplementation in rust), as it allows us to 
        // improve by 2.5 to 4% the throughput of the whole program for AVX2 256bit/8 lane vectors
        // when compared to the native & naïve implementation of `simd_atan2` which delegates to scalar calls of
        // `atan2`.
        //
        // This is AFTER the implementation of the similar optimization in [cc_per_node_face.rs]'s line 172,
        // where this optimization happened on acos
        //
        // This affects the perforamnce of (original -> per_node_face optim -> per_crease_fold_angle ULP 1.0)
        // - stepping/step_thirteen_horns_1_step: 14.759Kelem/s -> 17.029Kelem/s -> 17.595 Kelem/s
        // - stepping/step_simple_100_step: 553.74 Kelem/s -> 638.54 Kelem/s -> 652.38 Kelem/s
        //
        // Just as the previous optimization, this was found out via intel V-Tune tests with stepping\step_thirteen_horns_1_step
        // on AVX2 256bit/8, showing as ~3.2% of the performance before the switch to SLEEF, and as before, disappearing from the
        // call graph afterwards.
        let fold_angle = simba::simd::Simd(sleef::f32x::atan2_u35(y.0, x.0));
        tracing::event!(tracing::Level::TRACE, "uncorrected fold angle {fold_angle:?} (y: {y:?}, x: {x:?}, crease_vector: {crease_vector:?})");

        let zero = simba::simd::Simd(SimdF32::splat(0.0));
        let tau = simba::simd::Simd(SimdF32::splat(core::f32::consts::TAU));

        // given diff = current - previous:
        //  delta = diff + (diff < 5) ? TAU + (diff > 5) ? -TAU 
        let delta = {
            let diff = fold_angle - simba::simd::Simd(*previous_fold_angles);

            let under = diff.simd_le(simba::simd::Simd(SimdF32::splat(-5.0)));
            let under_diff = tau.select(under, zero);

            let over = diff.simd_ge(simba::simd::Simd(SimdF32::splat(5.0)));
            let over_diff = (-tau).select(over, zero);

            diff + under_diff + over_diff
        };

        let corrected_fold_angle = simba::simd::Simd(*previous_fold_angles) + delta;
        tracing::event!(tracing::Level::TRACE, "corrected fold angle {corrected_fold_angle:?} (diff: {delta:?})");

        corrected_fold_angle.0
    })
}
