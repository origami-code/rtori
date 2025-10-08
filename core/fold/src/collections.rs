pub use serde_seeded;

use itertools::Itertools;

pub type Index = u32;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(
    feature = "bytemuck",
    derive(bytemuck::NoUninit, bytemuck::AnyBitPattern)
)]
#[repr(transparent)]
pub struct MaskableIndex(pub Index);

impl MaskableIndex {
    const EMPTY_INNER: Index = Index::MAX;
    pub const EMPTY: Self = Self(Self::EMPTY_INNER);
}

impl From<Option<Index>> for MaskableIndex {
    fn from(value: Option<Index>) -> Self {
        value.map(|inner| Self(inner)).unwrap_or(Self::EMPTY)
    }
}

impl From<Index> for MaskableIndex {
    fn from(value: Index) -> Self {
        Self(value)
    }
}

impl Into<Option<Index>> for MaskableIndex {
    fn into(self) -> Option<Index> {
        match self.0 {
            Self::EMPTY_INNER => None,
            value => Some(value),
        }
    }
}

impl core::default::Default for MaskableIndex {
    fn default() -> Self {
        Self::EMPTY
    }
}

impl serde::Serialize for MaskableIndex {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        if self.0 == MaskableIndex::EMPTY_INNER {
            serializer.serialize_none()
        } else {
            serializer.serialize_u32(self.0)
        }
    }
}

impl<'de> serde::Deserialize<'de> for MaskableIndex {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct MaskableIndexVisitor;

        impl<'de> serde::de::Visitor<'de> for MaskableIndexVisitor {
            type Value = MaskableIndex;

            fn expecting(&self, formatter: &mut core::fmt::Formatter) -> core::fmt::Result {
                formatter.write_str("a u32 or null")
            }

            fn visit_u32<E>(self, value: u32) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(MaskableIndex(value))
            }

            fn visit_none<E>(self) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(MaskableIndex::EMPTY)
            }

            fn visit_unit<E>(self) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(MaskableIndex::EMPTY)
            }
        }

        deserializer.deserialize_option(MaskableIndexVisitor)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
#[cfg_attr(
    feature = "bytemuck",
    derive(bytemuck::NoUninit, bytemuck::AnyBitPattern)
)]
#[repr(transparent)]
pub struct MaskableFloat(pub f32);

impl MaskableFloat {
    const EMPTY_INNER: f32 = f32::NAN;
    pub const EMPTY: Self = Self(Self::EMPTY_INNER);
}

impl From<Option<f32>> for MaskableFloat {
    fn from(value: Option<f32>) -> Self {
        value
            .filter(|val| !val.is_nan())
            .map(|inner| Self(inner))
            .unwrap_or(Self::EMPTY)
    }
}

impl From<f32> for MaskableFloat {
    fn from(value: f32) -> Self {
        Some(value)
            .filter(|val| !val.is_nan())
            .map(|inner| Self(inner))
            .unwrap_or(Self::EMPTY)
    }
}

impl Into<Option<f32>> for MaskableFloat {
    fn into(self) -> Option<f32> {
        if !self.0.is_nan() {
            Some(self.0)
        } else {
            None
        }
    }
}

impl core::default::Default for MaskableFloat {
    fn default() -> Self {
        Self::EMPTY
    }
}

impl serde::Serialize for MaskableFloat {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        if self.0.is_nan() {
            serializer.serialize_none()
        } else {
            serializer.serialize_f32(self.0)
        }
    }
}

impl<'de> serde::Deserialize<'de> for MaskableFloat {
    fn deserialize<D>(deserializer: D) -> Result<MaskableFloat, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct MaskableFloatVisitor;

        impl<'de> serde::de::Visitor<'de> for MaskableFloatVisitor {
            type Value = MaskableFloat;

            fn expecting(&self, formatter: &mut core::fmt::Formatter) -> core::fmt::Result {
                formatter.write_str("a float (f32/f64) or null")
            }

            fn visit_f32<E>(self, value: f32) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(MaskableFloat::from(value))
            }

            fn visit_f64<E>(self, value: f64) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(MaskableFloat::from(value as f32))
            }

            fn visit_none<E>(self) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(MaskableFloat::EMPTY)
            }

            fn visit_unit<E>(self) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(MaskableFloat::EMPTY)
            }
        }

        deserializer.deserialize_option(MaskableFloatVisitor)
    }
}

#[derive(
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Clone,
    Default,
    Debug,
    derive_more::Display,
    derive_more::Deref,
    derive_more::DerefMut,
    serde::Serialize,
)]
#[serde(transparent)]
#[repr(transparent)]
pub struct String<Alloc>(pub string_alloc::String<Alloc>)
where
    Alloc: core::alloc::Allocator;

#[derive(
    Debug,
    Clone,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    derive_more::Deref,
    derive_more::DerefMut,
    serde::Serialize,
)]
#[serde(transparent)]
#[repr(transparent)]
pub struct SeededOption<T>(pub core::option::Option<T>);

impl<T> Default for SeededOption<T> {
    fn default() -> Self {
        Self(None)
    }
}

pub trait AsSlice {
    type Slice<'a>
    where
        Self: 'a;

    fn as_slice<'a>(&'a self) -> Self::Slice<'a>;
}

/// Uniform lockstep vector
#[derive(
    Debug, Clone, PartialEq, Eq, PartialOrd, Ord, derive_more::Deref, derive_more::DerefMut,
)]
#[repr(transparent)]
pub struct VecU<T, Alloc: core::alloc::Allocator>(pub alloc::vec::Vec<T, Alloc>);

impl<T, Alloc> core::default::Default for VecU<T, Alloc>
where
    Alloc: core::alloc::Allocator + core::default::Default,
{
    fn default() -> Self {
        Self(alloc::vec::Vec::new_in(Alloc::default()))
    }
}

impl<T, Alloc> serde::Serialize for VecU<T, Alloc>
where
    T: serde::Serialize,
    Alloc: core::alloc::Allocator,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeSeq;
        let mut seq = serializer.serialize_seq(Some(self.len()))?;
        for e in self.iter() {
            seq.serialize_element(e)?;
        }
        seq.end()
    }
}

impl<T, Alloc> AsSlice for VecU<T, Alloc>
where
    Alloc: core::alloc::Allocator,
{
    type Slice<'a>
        = &'a [T]
    where
        T: 'a,
        Alloc: 'a;

    fn as_slice<'a>(&'a self) -> Self::Slice<'a> {
        self.0.as_slice()
    }
}

#[derive(
    Debug,
    Clone,
    Copy,
    Default,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    serde::Serialize,
    serde::Deserialize,
)]
#[cfg_attr(
    feature = "bytemuck",
    derive(bytemuck::AnyBitPattern, bytemuck::NoUninit)
)]
#[repr(C)]
pub struct VecNURange {
    pub idx: u32,
    pub count: u32,
}

/// Non-uniform lockstep
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct VecNU<T, Alloc>
where
    Alloc: core::alloc::Allocator,
{
    pub backing: alloc::vec::Vec<T, Alloc>,
    pub indices: alloc::vec::Vec<VecNURange, Alloc>,
}

impl<T, Alloc> VecNU<T, Alloc>
where
    Alloc: core::alloc::Allocator,
{
    pub fn len(&self) -> usize {
        self.indices.len()
    }

    pub fn get(&self, idx: usize) -> Option<&[T]> {
        self.indices.get(idx).map(|range| {
            &self.backing.as_slice()[(range.idx as usize)..((range.idx + range.count) as usize)]
        })
    }

    /// It is possible for the non-uniform vector to actually house a perfectly
    /// uniform dataset.
    pub fn uniform_size(&self) -> Option<u32> {
        #[derive(Debug, Clone, Copy)]
        enum State {
            Standby,
            UniformForNow(u32),
            NotUniform,
        }

        let state = self
            .indices
            .iter()
            .fold_while(State::Standby, |state, el| match state {
                State::Standby => itertools::FoldWhile::Continue(State::UniformForNow(el.count)),
                State::UniformForNow(n) if n != el.count => {
                    itertools::FoldWhile::Done(State::NotUniform)
                }
                State::UniformForNow(_n) => itertools::FoldWhile::Continue(state),
                State::NotUniform => panic!("should have short-circuited"),
            })
            .into_inner();

        match state {
            State::Standby | State::NotUniform => None,
            State::UniformForNow(n) => Some(n),
        }
    }

    /// Attempts to flatten the non-uniform vector, checking if every
    /// entry is the same length, and if so, discards the non-uniform wrapper
    /// around the indices.
    #[cfg(feature = "bytemuck")]
    pub fn flatten_n_ref<const N: usize>(&self) -> core::option::Option<&'_ [[T; N]]>
    where
        T: bytemuck::Pod,
    {
        if self.indices.iter().any(|range| range.count != (N as u32)) {
            return None;
        }

        // I'm sure of the length, so I accept the panic behaviour of cast_slice if it was not to match
        let casted = bytemuck::cast_slice::<_, [T; N]>(self.backing.as_slice());
        Some(casted)
    }
}

impl<T, Alloc> core::default::Default for VecNU<T, Alloc>
where
    Alloc: core::alloc::Allocator + core::default::Default,
{
    fn default() -> Self {
        Self {
            backing: alloc::vec::Vec::new_in(Alloc::default()),
            indices: alloc::vec::Vec::new_in(Alloc::default()),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct NUSlice<'a, T> {
    backing: &'a [T],

    // the indices get slices, the backing never
    indices: &'a [VecNURange],
}

impl<'a, T> NUSlice<'a, T> {
    pub const fn new() -> Self {
        Self {
            backing: &[],
            indices: &[],
        }
    }

    const fn from_vec<Alloc>(src: &'a VecNU<T, Alloc>) -> Self
    where
        Alloc: core::alloc::Allocator,
    {
        Self {
            backing: &src.backing.as_slice(),
            indices: &src.indices.as_slice(),
        }
    }

    pub const fn len(&self) -> usize {
        self.indices.len()
    }
}

impl<'a, T> core::default::Default for NUSlice<'a, T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T, Alloc> AsSlice for VecNU<T, Alloc>
where
    Alloc: core::alloc::Allocator,
{
    type Slice<'a>
        = NUSlice<'a, T>
    where
        T: 'a,
        Alloc: 'a;

    fn as_slice<'a>(&'a self) -> Self::Slice<'a> {
        NUSlice::from_vec(self)
    }
}

/// Created by [`VecNU::into_iter`]
#[derive(Debug, Clone, Copy)]
pub struct NUIterator<'a, T> {
    backing: &'a [T],
    indices: &'a [VecNURange],
    index: usize,
}

impl<'a, T> Iterator for NUIterator<'a, T> {
    type Item = &'a [T];

    fn nth(&mut self, n: usize) -> core::option::Option<Self::Item> {
        self.index = self.index.saturating_add(n);

        let indices = self.indices.get(self.index)?;
        self.index += 1; // we still have to consume it

        let start =
            usize::try_from(indices.idx).expect("start index for given element overflows usize");
        let end = usize::try_from(indices.idx + indices.count)
            .expect("end index for given element overflows usize");
        Some(self.backing.get(start..end).expect(
            "an invalid source NUVec was provided: the indices point to non-existent backing",
        ))
    }

    fn next(&mut self) -> core::option::Option<Self::Item> {
        self.nth(0)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let size = self.indices.len().saturating_sub(self.index);
        (size, Some(size))
    }

    fn count(self) -> usize {
        self.len()
    }

    fn last(mut self) -> Option<Self::Item> {
        self.nth(self.len().saturating_sub(1))
    }
}

impl<T> ExactSizeIterator for NUIterator<'_, T> {}
impl<T> core::iter::FusedIterator for NUIterator<'_, T> {}

impl<'a, T> IntoIterator for NUSlice<'a, T> {
    type Item = &'a [T];
    type IntoIter = NUIterator<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        NUIterator {
            backing: self.backing,
            indices: self.indices,
            index: 0,
        }
    }
}
