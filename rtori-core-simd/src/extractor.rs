use core::{
    marker::PhantomData,
    simd::{LaneCount, SimdElement, SupportedLaneCount},
};

use rtori_os_model::ModelSize;

pub struct Extractor<'backer, const L: usize>
where
    LaneCount<L>: SupportedLaneCount,
{
    inner: &'backer crate::model::State<'backer, L>,
    size: ModelSize,
}

impl<'backer, const L: usize> Extractor<'backer, L>
where
    LaneCount<L>: SupportedLaneCount,
{
    pub const fn new(inner: &'backer crate::model::State<'backer, L>) -> Self {
        let size = inner.size();
        Self { inner, size }
    }
}

#[inline]
const fn scope_access<const L: usize, T>(slice: &[T], index: usize) -> (&T, usize) {
    let struct_index = index / L;
    let inner_index = index % L;

    let target = &slice[struct_index];

    (target, inner_index)
}

#[inline]
fn get_scalar<T, const L: usize>(slice: &[core::simd::Simd<T, L>], index: usize) -> T
where
    T: SimdElement,
    LaneCount<L>: SupportedLaneCount,
{
    let (target, inner_index) = scope_access::<L, _>(slice, index);
    target[inner_index]
}

#[inline]
fn get_vec<T, const N: usize, const L: usize>(
    slice: &[[core::simd::Simd<T, L>; N]],
    index: usize,
) -> [T; N]
where
    T: SimdElement + Default,
    LaneCount<L>: SupportedLaneCount,
{
    let (target, inner_index) = scope_access::<L, _>(slice, index);
    let mut value = [T::default(); N];
    for i in 0..N {
        value[i] = target[i][inner_index];
    }
    value
}

#[inline]
fn get_f32<const L: usize>(slice: &&[core::simd::Simd<f32, L>], index: usize) -> f32
where
    LaneCount<L>: SupportedLaneCount,
{
    get_scalar(slice, index)
}

#[inline]
fn get_vec3f<const L: usize>(
    slice: &&[[core::simd::Simd<f32, L>; 3]],
    index: usize,
) -> rtori_os_model::Vector3F
where
    LaneCount<L>: SupportedLaneCount,
{
    let inner = get_vec(slice, index);
    rtori_os_model::Vector3F(inner)
}

pub struct DummyMapped<T>(PhantomData<T>);

impl<T> core::ops::Deref for DummyMapped<T> {
    type Target = [T];

    fn deref(&self) -> &Self::Target {
        unimplemented!()
    }
}

pub struct LoaderReadAccess<'a, const L: usize, U, D, F> {
    data: D,
    len: usize,
    getter: F,
    _marker: PhantomData<&'a U>,
}

impl<'a, const L: usize, U, D, F> rtori_os_model::ReadAccess<'a, U>
    for LoaderReadAccess<'a, L, U, D, F>
where
    LaneCount<L>: SupportedLaneCount,
    F: Fn(&D, usize) -> U + 'a,
    U: Copy,
    D: 'a,
{
    fn get(&self, index: u32) -> U {
        (self.getter)(&self.data, index as usize)
    }

    fn count(&self) -> usize {
        self.len
    }

    fn copy_out(&self, out: &mut [U], offset: usize) {
        for (i, v) in out.into_iter().enumerate() {
            *v = (self.getter)(&self.data, i + offset);
        }
    }

    type Mapped<'b>
        = DummyMapped<U>
    where
        Self: 'b,
        'b: 'a;
}

impl<'backer, const L: usize> rtori_os_model::Extractor<'backer> for Extractor<'backer, L>
where
    LaneCount<L>: SupportedLaneCount,
{
    fn count_nodes(&self) -> usize {
        self.size.nodes as usize
    }

    type NodePositionAccess<'a>
        = LoaderReadAccess<
        'a,
        L,
        rtori_os_model::Vector3F,
        &'a [crate::simd_atoms::SimdVec3F<L>],
        impl Fn(&&'a [crate::simd_atoms::SimdVec3F<L>], usize) -> rtori_os_model::Vector3F,
    >
    where
        Self: 'a,
        'a: 'backer;

    fn access_node_position<'call>(&'call self) -> Option<Self::NodePositionAccess<'call>>
    where
        'call: 'backer,
    {
        Some(LoaderReadAccess {
            data: &self.inner.node_position_offset.back,
            len: self.count_nodes(),
            getter: get_vec3f,
            _marker: PhantomData,
        })
    }

    type NodeVelocityAccess<'a>
        = LoaderReadAccess<
        'a,
        L,
        rtori_os_model::Vector3F,
        &'a [crate::simd_atoms::SimdVec3F<L>],
        impl Fn(&&'a [crate::simd_atoms::SimdVec3F<L>], usize) -> rtori_os_model::Vector3F,
    >
    where
        Self: 'a,
        'a: 'backer;

    fn access_node_velocity<'call>(&'call self) -> Option<Self::NodeVelocityAccess<'call>>
    where
        'call: 'backer,
    {
        Some(LoaderReadAccess {
            data: &self.inner.node_velocity.back,
            len: self.count_nodes(),
            getter: get_vec3f,
            _marker: PhantomData,
        })
    }

    type NodeErrorAccess<'a>
        = LoaderReadAccess<
        'a,
        L,
        f32,
        &'a [crate::simd_atoms::SimdF32<L>],
        impl Fn(&&'a [crate::simd_atoms::SimdF32<L>], usize) -> f32,
    >
    where
        Self: 'a,
        'a: 'backer;

    fn access_node_error<'call>(&'call self) -> Option<Self::NodeErrorAccess<'call>>
    where
        'call: 'backer,
    {
        Some(LoaderReadAccess {
            data: &self.inner.node_error.0,
            len: self.count_nodes(),
            getter: get_f32,
            _marker: PhantomData,
        })
    }
}
