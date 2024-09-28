use core::simd::{LaneCount, SimdElement, SupportedLaneCount};

pub struct Loader<'backer, const L: usize>
where
    LaneCount<L>: SupportedLaneCount,
{
    inner: &'backer mut crate::model::State<'backer, L>,
}

impl<'backer, const L: usize> Loader<'backer, L>
where
    LaneCount<L>: SupportedLaneCount,
{
    pub const fn new(inner: &'backer mut crate::model::State<'backer, L>) -> Self {
        Self { inner }
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

impl<'backer, const L: usize> rtori_os_model::Destination for Loader<'backer, L>
where
    LaneCount<L>: SupportedLaneCount,
{
    #[inline]
    fn set_node_position(
        &mut self,
        node_idx: rtori_os_model::NodeIndex,
        pos: rtori_os_model::Vector3F,
    ) {
        set_vec(
            &mut self.inner.node_positions_unchanging,
            usize::try_from(node_idx).unwrap(),
            pos.0,
        )
    }

    #[inline]
    fn set_node_external_forces(
        &mut self,
        node_idx: rtori_os_model::NodeIndex,
        pos: rtori_os_model::Vector3F,
    ) {
        set_vec(
            &mut self.inner.node_external_forces,
            usize::try_from(node_idx).unwrap(),
            pos.0,
        )
    }

    #[inline]
    fn set_node_config(
        &mut self,
        node_idx: rtori_os_model::NodeIndex,
        config: rtori_os_model::NodeConfig,
    ) {
        set_scalar(
            &mut self.inner.node_mass,
            usize::try_from(node_idx).unwrap(),
            config.mass,
        );

        set_scalar(
            &mut self.inner.node_fixed,
            usize::try_from(node_idx).unwrap(),
            config.fixed.into(),
        );
    }

    #[inline]
    fn set_node_geometry(
        &mut self,
        node_idx: rtori_os_model::NodeIndex,
        geometry: rtori_os_model::NodeGeometry,
    ) {
        let (target, inner_index) = scope_access::<L, _>(
            &mut self.inner.node_geometry,
            usize::try_from(node_idx).unwrap(),
        );

        target.beams.offset[inner_index] = geometry.beam.offset;
        target.beams.count[inner_index] = geometry.beam.count;

        target.creases.offset[inner_index] = geometry.crease.offset;
        target.creases.count[inner_index] = geometry.crease.count;

        target.faces.offset[inner_index] = geometry.face.offset;
        target.faces.count[inner_index] = geometry.face.count;
    }

    #[inline]
    fn set_crease_geometry(
        &mut self,
        crease_idx: rtori_os_model::CreaseIndex,
        geometry: rtori_os_model::CreaseGeometry,
    ) {
        let crease_idx = usize::try_from(crease_idx).unwrap();

        let (target_complement, inner_index) =
            scope_access::<L, _>(&mut self.inner.crease_neighbourhoods, crease_idx);

        let (target_face_indices, _) =
            scope_access::<L, _>(&mut self.inner.crease_face_indices, crease_idx);

        for i in 0..2 {
            let face_spec = geometry.faces[i];
            target_complement.complement_node_indices[i][inner_index] =
                face_spec.complement_vertex_index;
            target_face_indices.0[i][inner_index] = face_spec.face_index;
        }
    }

    #[inline]
    fn set_crease_parameters(
        &mut self,
        crease_idx: rtori_os_model::CreaseIndex,
        parameters: rtori_os_model::CreaseParameters,
    ) {
        let crease_idx = usize::try_from(crease_idx).unwrap();

        set_scalar(&mut self.inner.crease_k, crease_idx, parameters.k);
        //set_scalar(&mut self.inner.crease_d, crease_idx, parameters.d);
        set_scalar(
            &mut self.inner.crease_target_fold_angle,
            crease_idx,
            parameters.target_fold_angle,
        );
    }

    #[inline]
    fn set_face_indices(
        &mut self,
        face_idx: rtori_os_model::FaceIndex,
        node_indices: rtori_os_model::Vector3U,
    ) {
        set_vec(
            &mut self.inner.face_indices,
            usize::try_from(face_idx).unwrap(),
            node_indices.0,
        )
    }

    #[inline]
    fn set_face_nominal_angles(
        &mut self,
        face_idx: rtori_os_model::FaceIndex,
        nominal_angles: rtori_os_model::Vector3F,
    ) {
        set_vec(
            &mut self.inner.face_nominal_angles,
            usize::try_from(face_idx).unwrap(),
            nominal_angles.0,
        )
    }

    #[inline]
    fn set_node_crease(
        &mut self,
        node_crease_idx: rtori_os_model::NodeCreaseIndex,
        spec: rtori_os_model::NodeCreaseSpec,
    ) {
        let node_crease_idx = usize::try_from(node_crease_idx).unwrap();

        set_scalar(
            &mut self.inner.node_crease_crease_indices,
            node_crease_idx,
            spec.crease_index,
        );

        set_scalar(
            &mut self.inner.node_crease_node_number,
            node_crease_idx,
            spec.node_number,
        );
    }

    #[inline]
    fn set_node_beam(
        &mut self,
        node_beam_idx: rtori_os_model::NodeBeamIndex,
        spec: rtori_os_model::NodeBeamSpec,
    ) {
        let node_beam_idx = usize::try_from(node_beam_idx).unwrap();

        {
            let (beam_spec_target, inner_index) =
                scope_access::<L, _>(&mut self.inner.node_beam_spec, node_beam_idx);
            beam_spec_target.node_indices[inner_index] = spec.node_index;
            beam_spec_target.neighbour_indices[inner_index] = spec.neighbour_index;
        }

        set_scalar(&mut self.inner.node_beam_length, node_beam_idx, spec.length);
        set_scalar(&mut self.inner.node_beam_k, node_beam_idx, spec.k);
        set_scalar(&mut self.inner.node_beam_d, node_beam_idx, spec.d);
    }

    #[inline]
    fn set_node_face(
        &mut self,
        node_face_idx: rtori_os_model::NodeFaceIndex,
        spec: rtori_os_model::NodeFaceSpec,
    ) {
        let node_face_idx = usize::try_from(node_face_idx).unwrap();

        let (node_face_spec_target, inner_index) =
            scope_access::<L, _>(&mut self.inner.node_face_spec, node_face_idx);
        node_face_spec_target.node_indices[inner_index] = spec.node_index;
        node_face_spec_target.face_indices[inner_index] = spec.face_index;
    }
}
