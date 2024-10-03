use core::{
    marker::PhantomData,
    simd::{LaneCount, SimdElement, SupportedLaneCount},
};

use rtori_os_model::ModelSize;

pub struct Loader<'backer, const L: usize>
where
    LaneCount<L>: SupportedLaneCount,
{
    inner: &'backer mut crate::model::State<'backer, L>,
    size: ModelSize,
}

impl<'backer, const L: usize> Loader<'backer, L>
where
    LaneCount<L>: SupportedLaneCount,
{
    pub const fn new(inner: &'backer mut crate::model::State<'backer, L>) -> Self {
        let size = inner.size();
        Self { inner, size }
    }
}

#[inline]
const fn scope_access<const L: usize, T>(slice: &mut [T], index: usize) -> (&mut T, usize) {
    let struct_index = index / L;
    let inner_index = index % L;

    let target = &mut slice[struct_index];

    (target, inner_index)
}

#[inline]
fn set_scalar<T, const L: usize>(slice: &mut [core::simd::Simd<T, L>], index: usize, value: T)
where
    T: SimdElement,
    LaneCount<L>: SupportedLaneCount,
{
    let (target, inner_index) = scope_access::<L, _>(slice, index);
    target[inner_index] = value;
}

#[inline]
fn set_vec<T, const N: usize, const L: usize>(
    slice: &mut [[core::simd::Simd<T, L>; N]],
    index: usize,
    value: [T; N],
) where
    T: SimdElement,
    LaneCount<L>: SupportedLaneCount,
{
    let (target, inner_index) = scope_access::<L, _>(slice, index);
    for i in 0..N {
        target[i][inner_index] = value[i];
    }
}

#[inline]
fn set_vec3f<const L: usize>(
    slice: &mut &mut [[core::simd::Simd<f32, L>; 3]],
    index: usize,
    value: rtori_os_model::Vector3F,
) where
    LaneCount<L>: SupportedLaneCount,
{
    set_vec(slice, index, value.0)
}

#[inline]
fn set_vec3u<const L: usize>(
    slice: &mut &mut [[core::simd::Simd<u32, L>; 3]],
    index: usize,
    value: rtori_os_model::Vector3U,
) where
    LaneCount<L>: SupportedLaneCount,
{
    set_vec(slice, index, value.0)
}

pub struct DummyMapped<T>(PhantomData<T>);

impl<T> core::ops::Deref for DummyMapped<T> {
    type Target = [T];

    fn deref(&self) -> &Self::Target {
        unimplemented!()
    }
}

impl<T> core::ops::DerefMut for DummyMapped<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unimplemented!()
    }
}

pub struct LoaderWriteAccess<'a, const L: usize, U, D, F> {
    data: D,
    len: usize,
    setter: F,
    _marker: PhantomData<&'a U>,
}

impl<'a, const L: usize, U, D, F> rtori_os_model::WriteAccess<'a, U>
    for LoaderWriteAccess<'a, L, U, D, F>
where
    LaneCount<L>: SupportedLaneCount,
    F: Fn(&mut D, usize, U) + 'a,
    U: Copy,
    D: 'a,
{
    fn capacity(&self) -> usize {
        self.len
    }

    fn copy_in(&mut self, from: &[U], offset: u32) {
        let offset = usize::try_from(offset).unwrap();

        // TODO: optimize by doing gather copies
        for (i, v) in from.into_iter().enumerate() {
            (self.setter)(&mut self.data, i + offset, *v)
        }
    }

    type Mapped<'b>
        = DummyMapped<U>
    where
        Self: 'b,
        'b: 'a;
}

pub type Vec3FAccess<'a, const L: usize>
where
    LaneCount<L>: SupportedLaneCount,
= LoaderWriteAccess<
    'a,
    L,
    rtori_os_model::Vector3F,
    &'a mut [crate::simd_atoms::SimdVec3F<L>],
    impl Fn(&mut &mut [crate::simd_atoms::SimdVec3F<L>], usize, rtori_os_model::Vector3F),
>;

pub type Vec3UAccess<'a, const L: usize>
where
    LaneCount<L>: SupportedLaneCount,
= LoaderWriteAccess<
    'a,
    L,
    rtori_os_model::Vector3U,
    &'a mut [crate::simd_atoms::SimdVec3U<L>],
    impl Fn(&mut &mut [crate::simd_atoms::SimdVec3U<L>], usize, rtori_os_model::Vector3U),
>;

impl<'loader, 'backer, const L: usize> rtori_os_model::Loader<'loader> for Loader<'backer, L>
where
    LaneCount<L>: SupportedLaneCount,
    'backer: 'loader,
{
    fn model(&self) -> rtori_os_model::ModelSize {
        self.size
    }

    type NodePositionAccess<'a>
        = Vec3FAccess<'a, L>
    where
        Self: 'a,
        'loader: 'a;

    fn access_node_position<'call, 'output>(&'call mut self) -> Self::NodePositionAccess<'output>
    where
        'call: 'output,
        'loader: 'output,
    {
        LoaderWriteAccess {
            data: &mut self.inner.node_positions_unchanging.data,
            len: self.size.nodes.try_into().unwrap(),
            setter: set_vec3f,
            _marker: PhantomData,
        }
    }

    type NodeExternalForcesAccess<'a>
        = Vec3FAccess<'a, L>
    where
        Self: 'a,
        'loader: 'a;

    fn access_node_external_forces<'call, 'output>(
        &'call mut self,
    ) -> Self::NodeExternalForcesAccess<'output>
    where
        'call: 'output,
        'loader: 'output,
    {
        LoaderWriteAccess {
            data: &mut self.inner.node_external_forces,
            len: self.size.nodes.try_into().unwrap(),
            setter: set_vec3f,
            _marker: PhantomData,
        }
    }

    type NodeConfigAccess<'a>
        = LoaderWriteAccess<
        'a,
        L,
        rtori_os_model::NodeConfig,
        (
            &'a mut [crate::simd_atoms::SimdF32<L>],
            &'a mut [crate::simd_atoms::SimdU32<L>],
        ),
        impl Fn(
            &mut (
                &'a mut [crate::simd_atoms::SimdF32<L>],
                &'a mut [crate::simd_atoms::SimdU32<L>],
            ),
            usize,
            rtori_os_model::NodeConfig,
        ),
    >
    where
        Self: 'a,
        'loader: 'a;

    fn access_node_config<'call, 'output>(&'call mut self) -> Self::NodeConfigAccess<'output>
    where
        'call: 'output,
        'loader: 'output,
    {
        #[inline]
        fn set_node_config<const L: usize>(
            data: &mut (
                &mut [core::simd::Simd<f32, L>],
                &mut [core::simd::Simd<u32, L>],
            ),
            index: usize,
            value: rtori_os_model::NodeConfig,
        ) where
            LaneCount<L>: SupportedLaneCount,
        {
            set_scalar(&mut data.0, index, value.mass);

            set_scalar(&mut data.1, index, value.fixed.into());
        }

        LoaderWriteAccess {
            data: (&mut self.inner.node_mass, &mut self.inner.node_fixed),
            len: self.size.nodes.try_into().unwrap(),
            setter: set_node_config,
            _marker: PhantomData,
        }
    }

    type NodeGeometryAccess<'a>
        = LoaderWriteAccess<
        'a,
        L,
        rtori_os_model::NodeGeometry,
        &'a mut [crate::model::NodeGeometry<L>],
        impl Fn(&mut &'a mut [crate::model::NodeGeometry<L>], usize, rtori_os_model::NodeGeometry),
    >
    where
        Self: 'a,
        'loader: 'a;

    fn access_node_geometry<'call, 'output>(&'call mut self) -> Self::NodeGeometryAccess<'output>
    where
        'call: 'output,
        'loader: 'output,
    {
        #[inline]
        fn set_node_geometry<const L: usize>(
            slice: &mut &mut [crate::model::NodeGeometry<L>],
            node_idx: usize,
            geometry: rtori_os_model::NodeGeometry,
        ) where
            LaneCount<L>: SupportedLaneCount,
        {
            let (target, inner_index) = scope_access::<L, _>(slice, node_idx);

            target.beams.offset[inner_index] = geometry.beam.offset;
            target.beams.count[inner_index] = geometry.beam.count;

            target.creases.offset[inner_index] = geometry.crease.offset;
            target.creases.count[inner_index] = geometry.crease.count;

            target.faces.offset[inner_index] = geometry.face.offset;
            target.faces.count[inner_index] = geometry.face.count;
        }

        LoaderWriteAccess {
            data: &mut self.inner.node_geometry.0,
            len: self.size.nodes.try_into().unwrap(),
            setter: set_node_geometry,
            _marker: PhantomData,
        }
    }

    type CreaseGeometryAccess<'a>
        = LoaderWriteAccess<
        'a,
        L,
        rtori_os_model::CreaseGeometry,
        (
            &'a mut [crate::model::CreaseNeighbourhood<L>],
            &'a mut [crate::model::CreaseFaceIndices<L>],
        ),
        impl Fn(
            &mut (
                &'a mut [crate::model::CreaseNeighbourhood<L>],
                &'a mut [crate::model::CreaseFaceIndices<L>],
            ),
            usize,
            rtori_os_model::CreaseGeometry,
        ),
    >
    where
        Self: 'a,
        'loader: 'a;

    fn access_crease_geometry<'call, 'output>(
        &'call mut self,
    ) -> Self::CreaseGeometryAccess<'output>
    where
        'call: 'output,
        'loader: 'output,
    {
        #[inline]
        fn set_crease_geometry<const L: usize>(
            data: &mut (
                &mut [crate::model::CreaseNeighbourhood<L>],
                &mut [crate::model::CreaseFaceIndices<L>],
            ),
            crease_idx: usize,
            geometry: rtori_os_model::CreaseGeometry,
        ) where
            LaneCount<L>: SupportedLaneCount,
        {
            let crease_idx = usize::try_from(crease_idx).unwrap();

            let (target_complement, inner_index) = scope_access::<L, _>(&mut data.0, crease_idx);

            let (target_face_indices, _) = scope_access::<L, _>(&mut data.1, crease_idx);

            for i in 0..2 {
                target_complement.complement_node_indices[i][inner_index] =
                    geometry.complementary_node_indices[i];
                target_face_indices.0[i][inner_index] = geometry.face_indices[i];
            }
        }

        LoaderWriteAccess {
            data: (
                &mut self.inner.crease_neighbourhoods,
                &mut self.inner.crease_face_indices,
            ),
            len: self.size.creases.try_into().unwrap(),
            setter: set_crease_geometry,
            _marker: PhantomData,
        }
    }

    type CreaseParametersAccess<'a>
        = LoaderWriteAccess<
        'a,
        L,
        rtori_os_model::CreaseParameters,
        (
            &'a mut [crate::simd_atoms::SimdF32<L>],
            &'a mut [crate::simd_atoms::SimdF32<L>],
        ),
        impl Fn(
            &mut (
                &'a mut [crate::simd_atoms::SimdF32<L>],
                &'a mut [crate::simd_atoms::SimdF32<L>],
            ),
            usize,
            rtori_os_model::CreaseParameters,
        ),
    >
    where
        Self: 'a,
        'loader: 'a;

    fn access_crease_parameters<'call, 'output>(
        &'call mut self,
    ) -> Self::CreaseParametersAccess<'output>
    where
        'call: 'output,
        'loader: 'output,
    {
        #[inline]
        fn set_crease_parameters<const L: usize>(
            data: &mut (
                &mut [crate::simd_atoms::SimdF32<L>],
                &mut [crate::simd_atoms::SimdF32<L>],
            ),
            crease_idx: usize,
            parameters: rtori_os_model::CreaseParameters,
        ) where
            LaneCount<L>: SupportedLaneCount,
        {
            set_scalar(&mut data.0, crease_idx, parameters.k);
            //set_scalar(&mut self.inner.crease_d, crease_idx, parameters.d);
            set_scalar(&mut data.1, crease_idx, parameters.target_fold_angle);
        }

        LoaderWriteAccess {
            data: (
                &mut self.inner.crease_k,
                &mut self.inner.crease_target_fold_angle,
            ),
            len: self.size.creases.try_into().unwrap(),
            setter: set_crease_parameters,
            _marker: PhantomData,
        }
    }

    type FaceIndicesAccess<'a>
        = Vec3UAccess<'a, L>
    where
        Self: 'a,
        'loader: 'a;

    fn access_face_indices<'call, 'output>(&'call mut self) -> Self::FaceIndicesAccess<'output>
    where
        'call: 'output,
        'loader: 'output,
    {
        LoaderWriteAccess {
            data: &mut self.inner.face_indices,
            len: self.size.faces.try_into().unwrap(),
            setter: set_vec3u,
            _marker: PhantomData,
        }
    }

    type FaceNominalAnglesAccess<'a>
        = Vec3FAccess<'a, L>
    where
        Self: 'a,
        'loader: 'a;

    fn access_face_nominal_angles<'call, 'output>(
        &'call mut self,
    ) -> Self::FaceNominalAnglesAccess<'output>
    where
        'call: 'output,
        'loader: 'output,
    {
        LoaderWriteAccess {
            data: &mut self.inner.face_nominal_angles,
            len: self.size.faces.try_into().unwrap(),
            setter: set_vec3f,
            _marker: PhantomData,
        }
    }

    type NodeCreaseAccess<'a>
        = LoaderWriteAccess<
        'a,
        L,
        rtori_os_model::NodeCreaseSpec,
        (
            &'a mut [crate::simd_atoms::SimdU32<L>],
            &'a mut [crate::simd_atoms::SimdU32<L>],
        ),
        impl Fn(
            &mut (
                &'a mut [crate::simd_atoms::SimdU32<L>],
                &'a mut [crate::simd_atoms::SimdU32<L>],
            ),
            usize,
            rtori_os_model::NodeCreaseSpec,
        ),
    >
    where
        Self: 'a,
        'loader: 'a;

    fn access_node_crease<'call, 'output>(&'call mut self) -> Self::NodeCreaseAccess<'output>
    where
        'call: 'output,
        'loader: 'output,
    {
        #[inline]
        fn set_node_crease<const L: usize>(
            data: &mut (
                &mut [crate::simd_atoms::SimdU32<L>],
                &mut [crate::simd_atoms::SimdU32<L>],
            ),
            node_crease_idx: usize,
            spec: rtori_os_model::NodeCreaseSpec,
        ) where
            LaneCount<L>: SupportedLaneCount,
        {
            set_scalar(&mut data.0, node_crease_idx, spec.crease_index);

            set_scalar(&mut data.1, node_crease_idx, spec.node_number);
        }

        LoaderWriteAccess {
            data: (
                &mut self.inner.node_crease_crease_indices,
                &mut self.inner.node_crease_node_number,
            ),
            len: self.size.node_creases.try_into().unwrap(),
            setter: set_node_crease,
            _marker: PhantomData,
        }
    }

    type NodeBeamAccess<'a>
        = LoaderWriteAccess<
        'a,
        L,
        rtori_os_model::NodeBeamSpec,
        (
            &'a mut [crate::model::NodeBeamSpec<L>],
            &'a mut [crate::simd_atoms::SimdF32<L>],
            &'a mut [crate::simd_atoms::SimdF32<L>],
            &'a mut [crate::simd_atoms::SimdF32<L>],
        ),
        impl Fn(
            &mut (
                &'a mut [crate::model::NodeBeamSpec<L>],
                &'a mut [crate::simd_atoms::SimdF32<L>],
                &'a mut [crate::simd_atoms::SimdF32<L>],
                &'a mut [crate::simd_atoms::SimdF32<L>],
            ),
            usize,
            rtori_os_model::NodeBeamSpec,
        ),
    >
    where
        Self: 'a,
        'loader: 'a;

    fn access_node_beam<'call, 'output>(&'call mut self) -> Self::NodeBeamAccess<'output>
    where
        'call: 'output,
        'loader: 'output,
    {
        #[inline]
        fn set_node_beam<const L: usize>(
            data: &mut (
                &mut [crate::model::NodeBeamSpec<L>],
                &mut [crate::simd_atoms::SimdF32<L>],
                &mut [crate::simd_atoms::SimdF32<L>],
                &mut [crate::simd_atoms::SimdF32<L>],
            ),
            node_beam_idx: usize,
            spec: rtori_os_model::NodeBeamSpec,
        ) where
            LaneCount<L>: SupportedLaneCount,
        {
            let (node_beam_spec, node_beam_length, node_beam_k, node_beam_d) = data;

            {
                let (beam_spec_target, inner_index) =
                    scope_access::<L, _>(node_beam_spec, node_beam_idx);
                beam_spec_target.node_indices[inner_index] = spec.node_index;
                beam_spec_target.neighbour_indices[inner_index] = spec.neighbour_index;
            }

            set_scalar(node_beam_length, node_beam_idx, spec.length);
            set_scalar(node_beam_k, node_beam_idx, spec.k);
            set_scalar(node_beam_d, node_beam_idx, spec.d);
        }

        LoaderWriteAccess {
            data: (
                &mut self.inner.node_beam_spec,
                &mut self.inner.node_beam_length,
                &mut self.inner.node_beam_k,
                &mut self.inner.node_beam_d,
            ),
            len: self.size.node_beams.try_into().unwrap(),
            setter: set_node_beam,
            _marker: PhantomData,
        }
    }

    type NodeFaceAccess<'a>
        = LoaderWriteAccess<
        'a,
        L,
        rtori_os_model::NodeFaceSpec,
        &'a mut [crate::model::NodeFaceSpec<L>],
        impl Fn(&mut &'a mut [crate::model::NodeFaceSpec<L>], usize, rtori_os_model::NodeFaceSpec),
    >
    where
        Self: 'a,
        'loader: 'a;

    fn access_node_face<'call, 'output>(&'call mut self) -> Self::NodeFaceAccess<'output>
    where
        'call: 'output,
        'loader: 'output,
    {
        #[inline]
        fn set_node_face<const L: usize>(
            data: &mut &mut [crate::model::NodeFaceSpec<L>],
            node_face_idx: usize,
            spec: rtori_os_model::NodeFaceSpec,
        ) where
            LaneCount<L>: SupportedLaneCount,
        {
            let (node_face_spec_target, inner_index) = scope_access::<L, _>(data, node_face_idx);
            node_face_spec_target.node_indices[inner_index] = spec.node_index;
            node_face_spec_target.face_indices[inner_index] = spec.face_index;
        }

        LoaderWriteAccess {
            data: &mut self.inner.node_face_spec.0,
            len: self.size.node_faces.try_into().unwrap(),
            setter: set_node_face,
            _marker: PhantomData,
        }
    }
}
