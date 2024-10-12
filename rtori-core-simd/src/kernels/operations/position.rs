use core::simd::{LaneCount, SupportedLaneCount};

use super::algebra::algebrize;
use super::gather::gather_vec3f;

use crate::simd_atoms::*;

use nalgebra as na;

#[inline]
pub fn get_positions_for_indices<const L: usize>(
    positions_unchanging: &[SimdVec3F<L>],
    positions_offsets: &[SimdVec3F<L>],
    indices: SimdU32<L>,
) -> na::Vector3<simba::simd::Simd<SimdF32<L>>>
where
    LaneCount<L>: SupportedLaneCount,
    simba::simd::Simd<core::simd::Simd<f32, L>>: num_traits::Num + num_traits::NumAssign,
{
    //println!("positions unchanging: {:?}", positions_unchanging);

    let [unchanging, offset] = gather_vec3f([positions_unchanging, positions_offsets], indices);

    let unchanging = algebrize(unchanging);
    let offset = algebrize(offset);

    let position = unchanging + offset;
    super::debug::check_nans_simd_vec_msg::<{ L }, 3>(
        [position.x.0, position.y.0, position.z.0],
        "get_positions_for_indices",
        "position",
    );

    /*2024-10-11*/
    /*println!(
        "get_positions_for_indices: indices {:?} => position {:?}",
        indices, position
    ); */
    position
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn test_position_for_indices() {
        const POSITIONS_UNCHANGING: [[f32; 3]; 4] = [
            [0.0f32, 0.0f32, 0.0f32],
            [1.0f32, 1.0f32, 1.0f32],
            [2.0f32, 2.0f32, 2.0f32],
            [3.0f32, 3.0f32, 3.0f32],
        ];

        const POSITIONS_OFFSETS: [[f32; 3]; 4] = [
            [0.0f32, 0.0f32, 0.0f32],
            [0.0f32, 0.0f32, 0.0f32],
            [0.0f32, 0.0f32, 0.0f32],
            [0.0f32, 0.0f32, 0.0f32],
        ];

        let positions_unchaging_simd = [0, 1, 2].map(|idx| {
            SimdF32::from_array([
                POSITIONS_UNCHANGING[0][idx],
                POSITIONS_UNCHANGING[1][idx],
                POSITIONS_UNCHANGING[2][idx],
                POSITIONS_UNCHANGING[3][idx],
            ])
        });

        let positions_offsets_simd = [0, 1, 2].map(|idx| {
            SimdF32::from_array([
                POSITIONS_OFFSETS[0][idx],
                POSITIONS_OFFSETS[1][idx],
                POSITIONS_OFFSETS[2][idx],
                POSITIONS_OFFSETS[3][idx],
            ])
        });

        let positions = get_positions_for_indices(
            &[positions_unchaging_simd],
            &[positions_offsets_simd],
            SimdU32::<4>::from_array([0, 1, 2, 3]),
        );
        // /*2024-10-11*/ println!("eq {positions} {positions_unchaging_simd:?}");

        for i in 0..4 {
            assert_eq!(positions.x.0[i], positions_unchaging_simd[0][i]);
            assert_eq!(positions.y.0[i], positions_unchaging_simd[1][i]);
            assert_eq!(positions.z.0[i], positions_unchaging_simd[2][i]);
        }
    }
}
