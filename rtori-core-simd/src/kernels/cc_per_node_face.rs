use core::simd::cmp::SimdPartialEq;
use core::simd::LaneCount;
use core::simd::SupportedLaneCount;
use nalgebra::SimdComplexField;

use super::algebra::algebrize;
use super::operations::debug::ensure_simd;
use super::position;
use crate::model::NodeFaceSpec;
use crate::simd_atoms::*;

#[derive(Debug)]
pub struct PerNodeFaceInput<'backer, const L: usize>
where
    LaneCount<L>: SupportedLaneCount,
{
    pub node_face_spec: &'backer [NodeFaceSpec<L>],

    pub node_positions_unchanging: &'backer [SimdVec3F<L>],
    pub node_positions_offset: &'backer [SimdVec3F<L>],
    pub node_velocity: &'backer [SimdVec3F<L>],

    pub face_node_indices: &'backer [SimdVec3U<L>],
    pub face_normals: &'backer [SimdVec3F<L>],
    pub face_nominal_angles: &'backer [SimdVec3F<L>],

    pub face_stiffness: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct PerNodeFaceOutput<const L: usize>
where
    LaneCount<L>: SupportedLaneCount,
{
    pub force: SimdVec3F<L>,
    pub error: SimdF32<L>,
}

const TOL: f32 = 0.0000001;

pub fn calculate_node_face_forces<'a, const L: usize>(
    inputs: &'a PerNodeFaceInput<'a, L>,
) -> impl ExactSizeIterator<Item = PerNodeFaceOutput<L>> + use<'a, L>
where
    LaneCount<L>: SupportedLaneCount,
    simba::simd::Simd<core::simd::Simd<f32, L>>: simba::simd::SimdRealField,
{
    let tol = SimdF32::splat(TOL);
    let face_stiffness = simba::simd::Simd(SimdF32::splat(inputs.face_stiffness));
    let zero = simba::simd::Simd(SimdF32::splat(0.0));
    let zero_force = nalgebra::Vector3::new(zero, zero, zero);

    inputs.node_face_spec.iter().map(
        #[inline]
        move |spec| {
            use super::operations::select;

            let face_vertex_indices =
                super::gather::gather_vec3([&inputs.face_node_indices], spec.face_indices)[0];

            // Unnormalized ab, ac & bc vectors, as well as their lengths
            let (
                [ab, ac, bc],
                [ab_length, ac_length, bc_length]
            ) = {
                // We have the node indices, so we don't need to do that weird magic
                let [a,b,c] = {
                    let a = position::get_positions_for_indices(
                        &inputs.node_positions_unchanging,
                        &inputs.node_positions_offset,
                        face_vertex_indices[0],
                    );
                    let b = position::get_positions_for_indices(
                        &inputs.node_positions_unchanging,
                        &inputs.node_positions_offset,
                        face_vertex_indices[1],
                    );
                    let c = position::get_positions_for_indices(
                        &inputs.node_positions_unchanging,
                        &inputs.node_positions_offset,
                        face_vertex_indices[2],
                    );

                    [a, b, c]
                };

                let [ab, ac, bc] = [
                    b - a,
                    c - a,
                    c - b
                ];

                let [ab_length, ac_length, bc_length] = [
                    ab.norm(),
                    ac.norm(),
                    bc.norm()
                ];

                (
                    [ab, ac, bc],
                    [ab_length, ac_length, bc_length]
                )
            };

            // /*2024-10-11*/ println!("per_node_face: ab {ab:?}, ac {ac:?}, bc {bc:?}");

            // Skip if lower than tolerance
            let mask = {
                use std::simd::cmp::SimdPartialOrd as _;
                let ab_mask = ab_length.0.simd_gt(tol);
                let ac_mask = ac_length.0.simd_gt(tol);
                let bc_mask = bc_length.0.simd_gt(tol);
                ab_mask & ac_mask & bc_mask
            };

            if (!mask).all() {
                return PerNodeFaceOutput {
                    force: [zero.0, zero.0, zero.0],
                    error: zero.0,
                };
            }
            
            // Normalize ab, ac & bc
            let [ab, ac, bc] = [
                ensure_simd!(ab / ab_length; v3; @depends(ab, ab_length)),
                ensure_simd!(ac / ac_length; v3; @depends(ac, ac_length)),
                ensure_simd!(bc / bc_length; v3; @depends(bc, bc_length))
            ];
            
            // /*2024-10-11*/ println!("per_node_face (normalized): ab {ab:?}, ac {ac:?}, bc {bc:?}");

            // Euleur angles
            use simba::simd::SimdPartialOrd; // for simd_min
            let angles = {
                // These are not yet the angles, but the values between 0 & 1, result of a dot product (directly or negated)
                // Unfortunately, sometimes, due to precision issues, the dot product gives values exceeding that range by miniscule values.
                // See the next operation on how we fix that
                let dot_products = [
                    ensure_simd!(
                        ab.dot(&ac);
                        sw;
                        @depends(ab, ac)
                    ), 
                    ensure_simd!(simba::simd::Simd(SimdF32::splat(-1.0)) * ab.dot(&bc); sw; @depends(ab, bc)),
                    ensure_simd!(ac.dot(&bc); sw; @depends(ac, bc)),
                ];

                // We have to ensure that the input to acos has, for each lane, a value between -1 and 1.
                // This input comes from the dot product (directly or negated).
                // Calling acos on these cause NaNs, which then propagate everywhere.
                // Thus, we clamp the value.
                let clamped = dot_products.map(|v| v.simd_clamp(
                    simba::simd::Simd(core::simd::Simd::splat(-1.0f32)),
                    simba::simd::Simd(core::simd::Simd::splat(1.0f32))
                ));

                // Finally, we can call acos
                let acos = clamped.map(|v| v.simd_acos());

                acos
            };
            // /*2024-10-11*/ println!("per_node_face angles: {angles:?}");

            let [normal, nominal_angles] = super::gather::gather_vec3f(
                [&inputs.face_normals, &inputs.face_nominal_angles],
                spec.face_indices,
            );
            ensure_simd!(normal; a);
            ensure_simd!(nominal_angles; a);

            let angles_diff =
                algebrize(nominal_angles) - nalgebra::Vector3::new(angles[0], angles[1], angles[2]);
            ensure_simd!(angles_diff; v3);
            /* 2025-01-15 */ //println!("cc_per_node_face: Angles Difference: {angles_diff:?}");

            let angles_diff_scaled = ensure_simd!(angles_diff.scale(face_stiffness); v3);

            let [
                is_a,
                is_b,
                is_c
            ] = face_vertex_indices.map(|index| spec.node_indices.simd_eq(index));

            let select_side = |sel, lhs, lhs_length: simba::simd::Simd<core::simd::Simd<f32, {L}>>, rhs, rhs_length: simba::simd::Simd<core::simd::Simd<f32, {L}>>| {
                let selected = ensure_simd!(select(sel, lhs, rhs); v3);
                let selected_length = simba::simd::Simd(sel.select(lhs_length.0, rhs_length.0));

                (selected, selected_length)
            };

            let (left, left_length) = select_side(is_b, ab, ab_length, ac, ac_length);
            let (right, right_length) = select_side(is_a, ab, ab_length, bc, bc_length);

            let normal = algebrize(normal);
            let cross_left = ensure_simd!(normal.cross(&left) / left_length; v3);
            let cross_right = ensure_simd!(normal.cross(&right) / right_length; v3);

            let force= {
                let force_a = select(
                    is_a,
                    {
                        let mut force = zero_force;
                        force -= (cross_left - cross_right) * angles_diff_scaled.x;
                        // TODO: face strain
                        force -= cross_right * angles_diff_scaled.y;
                        force += cross_left * angles_diff_scaled.z;
                        force
                    },
                    zero_force,
                );
                ensure_simd!(force_a; v3);

                let force_b = select(
                    is_b,
                    {
                        let mut force = zero_force;
                        force -= left * angles_diff_scaled.x;
                        force += (left + right) * angles_diff_scaled.y;
                        // TODO: face strain
                        force -= right * angles_diff_scaled.z;
                        force
                    },
                    zero_force,
                );
                ensure_simd!(force_b; v3);

                let force_c = select(
                    is_c,
                    {
                        let mut force = zero_force;
                        force += left * angles_diff.x;
                        force -= right * angles_diff.y;
                        force += (right - left) * angles_diff.z;
                        force
                    },
                    zero_force,
                );
                ensure_simd!(force_c; v3);

                let force = ensure_simd!(force_a + force_b + force_c; v3);
                force
            };
            let error = zero;

            let force_selected =  ensure_simd!([
                mask.select(force.x.0, zero.0),
                mask.select(force.y.0, zero.0),
                mask.select(force.z.0, zero.0),
            ]; a);

            let error_selected = mask.select(error.0, zero.0);

            let output = PerNodeFaceOutput {
                force: force_selected,
                error: error_selected,
            };
            // /*2024-10-11*/ println!("per_node_face: {output:?}");
            output
        },
    )
}
