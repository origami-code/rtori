// #![no_std] temporary for testing
#![feature(portable_simd)]
#![feature(iter_array_chunks)]
#![feature(stmt_expr_attributes)]
#![feature(impl_trait_in_assoc_type)]
#![feature(type_alias_impl_trait)]
#![cfg_attr(feature = "alloc", feature(allocator_api))]

use core::simd::{LaneCount, SupportedLaneCount};

extern crate static_assertions;

mod kernels;
mod model;
mod process;
mod simd_atoms;
pub use simd_atoms::PREFERRED_WIDTH;
#[cfg(feature = "alloc")]
pub mod owned;

mod extractor;
pub use extractor::Extractor;

mod loader;
pub use loader::Loader;
#[derive(Debug)]
pub struct Runner<'backer, const L: usize = { simd_atoms::PREFERRED_WIDTH }>
where
    LaneCount<L>: SupportedLaneCount,
{
    steps: u64,
    state: model::State<'backer, L>,
}

impl<'backer, const L: usize> Runner<'backer, L>
where
    LaneCount<L>: SupportedLaneCount,
    simba::simd::Simd<core::simd::Simd<f32, L>>: simba::simd::SimdRealField,
{
    /// The number of steps done
    pub fn steps(&self) -> u64 {
        self.steps
    }

    #[tracing::instrument]
    pub fn step(&mut self) -> Result<(), ()> {
        let state = &mut self.state;

        let input = process::ReadOnlyInput {
            node_geometry: &state.node_geometry,
            node_positions_unchanging: &state.node_positions_unchanging,
            node_external_forces: &state.node_external_forces,
            node_mass: &state.node_mass,
            node_fixed: &state.node_fixed,
            node_position_offset: state.node_position_offset.back,
            node_velocity: state.node_velocity.back,
            crease_face_indices: &state.crease_face_indices,
            crease_neighbourhoods: &state.crease_neighbourhoods,
            crease_k: &state.crease_k,
            crease_target_fold_angle: &state.crease_target_fold_angle,
            crease_fold_angle: state.crease_fold_angle.back,
            face_indices: &state.face_indices,
            face_nominal_angles: &state.face_nominal_angles,
            node_crease_crease_indices: &state.node_crease_crease_indices,
            node_crease_node_number: &state.node_crease_node_number,
            node_beam_spec: &state.node_beam_spec,
            node_beam_length: &state.node_beam_length,
            node_beam_k: &state.node_beam_k,
            node_beam_d: &state.node_beam_d,
            node_face_spec: &state.node_face_spec,
            crease_percentage: state.crease_percentage,
            dt: state.dt,
            face_stiffness: state.face_stiffness,
        };

        let mut scratch = process::ScratchInput {
            crease_physics: &mut state.crease_physics,
            face_normals: &mut state.face_normals,
            node_crease_forces: &mut state.node_crease_forces,
            node_beam_forces: &mut state.node_beam_forces,
            node_beam_error: &mut state.node_beam_error,
            node_face_forces: &mut state.node_face_forces,
            node_face_error: &mut state.node_face_error,
        };

        let mut memorable = process::MemorableInput {
            crease_fold_angle: &mut state.crease_fold_angle.front,
        };

        let it = process::process(&input, &mut scratch, &mut memorable);

        let position_dest = &mut state.node_position_offset.front;
        assert!(it.len() <= position_dest.len());

        let velocity_dest = &mut state.node_velocity.front;
        assert!(it.len() <= velocity_dest.len());

        let error_dest = &mut state.node_error;
        assert!(it.len() <= error_dest.len());

        for (i, output) in it.enumerate() {
            position_dest[i] = output.position_offset;
            velocity_dest[i] = output.velocity;
            error_dest[i] = output.error;
        }

        // Swap
        state.node_position_offset.swap();
        state.node_velocity.swap();
        state.crease_fold_angle.swap();

        self.steps += 1;

        Ok(())
    }

    pub fn query_backing_size_requirement(sizes: &rtori_os_model::ModelSize) -> usize {
        crate::model::State::required_backing_size(sizes)
    }

    /// Returns what is unused
    pub fn from_backing_slice(
        sizes: &rtori_os_model::ModelSize,
        backing_slice: &'backer mut [u8],
    ) -> Result<(Self, &'backer mut [u8]), usize> {
        crate::model::State::from_slice(sizes, backing_slice)
            .map_err(|_| crate::model::State::required_backing_size(sizes))
            .map(|(state, rest)| (Self { steps: 0, state }, rest))
    }

    pub fn from_allocator_func<F>(
        sizes: &rtori_os_model::ModelSize,
        allocator: F,
    ) -> Option<(Self, &'backer mut [u8])>
    where
        F: FnOnce(usize) -> Option<&'backer mut [u8]>,
    {
        // Calculate how much size it would take
        let size = crate::model::State::required_backing_size(sizes);
        let backing_array = allocator(size)?;

        Self::from_backing_slice(sizes, backing_array)
            .expect("backing_array should explicitely be of the right size, or larger")
            .into()
    }

    pub fn load<'a>(&'a mut self) -> crate::loader::Loader<'a, 'backer, L> {
        loader::Loader::new(self)
    }

    pub fn set_fold_percentage(&mut self, fold_percentage: f32) {
        self.state.crease_percentage = fold_percentage;
    }

    pub fn compute_dt(&mut self) {
        /*
                const naturalFrequencies = input.edges_vertices
                    .map((_, edgeIndex) => FoldOp.calcNaturalFrequency(input, edgeIndex, this.#axialStiffness, this.#globalMass));

                const maximumNaturalFrequency = Math.max(
                    ...naturalFrequencies
                );

                // original note says:
                // 0.9 of max delta t for good measure
                const dt = (1.0 / (2.0 * Math.PI * maximumNaturalFrequency)) * 0.9;
                return dt;


                knowing

                export function calcAxialK<V extends number, E extends number, F extends number>(input: Fold<V, E, F>, edgeIndex: number, defaultStiffness: number): number {
            const axialStiffness = getAxialStiffness(input, edgeIndex) ?? defaultStiffness;
            const length = calcLength(input, edgeIndex);
            return axialStiffness / length;
        }

        export function calcMinimumMass<V extends number, E extends number, F extends number>(input: Fold<V, E, F>, edgeIndex: number, defaultMass: number): number {
            const nodeMasses = input.edges_vertices[edgeIndex]
                .flat().map((v) => getNodeMass<V, E, F>(input, v) ?? defaultMass) as FixedArray<number, 2>;
            const minMass = Math.min(nodeMasses[0], nodeMasses[1]);
            return minMass;
        }

        export function calcNaturalFrequency<V extends number, E extends number, F extends number>(input: Fold<V, E, F>, edgeIndex: number, defaultStiffness: number, defaultMass: number): number {
            const k = calcAxialK(input, edgeIndex, defaultStiffness);
            const minMass = calcMinimumMass(input, edgeIndex, defaultMass);
            return Math.sqrt(k / minMass);
        }
                */
    }

    pub fn extract<'a>(
        &'a self,
        _flags: rtori_os_model::ExtractFlags,
    ) -> extractor::Extractor<'a, L>
    where
        'backer: 'a,
    {
        extractor::Extractor::new(&self.state)
    }
}
