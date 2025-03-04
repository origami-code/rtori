use super::InputWithCreaseGeometry;
use alloc::alloc::Allocator;

impl<'input, I, A> InputWithCreaseGeometry<'input, I, A>
where
    A: Allocator,
{
    pub fn compute_dt(&self) -> f32 {
        //self.preprocessed.creases
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
         */

        todo!()
    }
}
