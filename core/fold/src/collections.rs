use itertools::Itertools;

#[derive(
    Debug,
    Clone,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    derive_more::Display,
    derive_more::Deref,
    derive_more::DerefMut,
    serde::Serialize,
)]
#[serde(transparent)]
#[repr(transparent)]
pub struct String<'alloc>(pub bumpalo::collections::String<'alloc>);

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

/// Uniform lockstep vector
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
pub struct VecU<'alloc, T>(pub bumpalo::collections::Vec<'alloc, T>);

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
pub struct VecNURange {
    pub idx: u32,
    pub count: u32,
}

/// Non-uniform lockstep
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct VecNU<'alloc, T> {
    pub backing: bumpalo::collections::Vec<'alloc, T>,
    pub indices: bumpalo::collections::Vec<'alloc, VecNURange>,
}

impl<'a, T> VecNU<'a, T> {
    pub fn len(&self) -> usize {
        self.indices.len()
    }

    pub fn get(&self, idx: usize) -> Option<&[T]> {
        self.indices.get(idx)
            .map( |range| &self.backing.as_slice()[(range.idx as usize)..((range.idx + range.count) as usize)])
    }

    /// It is possible for the non-uniform vector to actually house a perfectly
    /// uniform dataset.
    pub fn uniform_size(&self) -> Option<u32> {
        #[derive(Debug, Clone, Copy)]
        enum State {
            Standby,
            UniformForNow(u32),
            NotUniform
        }

        let state = self.indices.iter().fold_while(State::Standby, |state, el| {
            match state {
                State::Standby => itertools::FoldWhile::Continue(State::UniformForNow(el.count)),
                State::UniformForNow(n) if n != el.count => itertools::FoldWhile::Done(State::NotUniform),
                State::UniformForNow(_n) => itertools::FoldWhile::Continue(state),
                State::NotUniform => panic!("should have short-circuited")
            }
        }).into_inner();

        match state {
            State::Standby | State::NotUniform => None,
            State::UniformForNow(n) => Some(n)
        }
    }
    
    /// Attempts to flatten the non-uniform vector, checking if every
    /// entry is the same length, and if so, discards the non-uniform wrapper
    /// around the indices.
    pub fn flatten_n_ref<const N: usize>(&self) -> core::option::Option<&'_ [[T; N]]> where T: bytemuck::Pod {
        if self.indices.iter().any(|range| range.count != (N as u32)) {
            return None;
        }

        // I'm sure of the length, so I accept the panic behaviour of cast_slice if it was not to match
        let casted = bytemuck::cast_slice::<_, [T; N]>(self.backing.as_slice());
        Some(casted)
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

        let start = usize::try_from(indices.idx).expect("start index for given element overflows usize");
        let end = usize::try_from(indices.idx + indices.count).expect("end index for given element overflows usize");
        Some(self.backing.get(start..end).expect("an invalid source NUVec was provided: the indices point to non-existent backing"))
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

impl<'alloc, T> IntoIterator for &'alloc VecNU<'alloc, T> {
    type Item = &'alloc [T];
    type IntoIter = NUIterator<'alloc, T>;

    fn into_iter(self) -> Self::IntoIter {
        NUIterator {
            backing: self.backing.as_slice(),
            indices: self.indices.as_slice(),
            index: 0,
        }
    }
}

pub type Handful<'alloc, T, const N: usize> = VecU<'alloc, T>;

pub type Lockstep<'alloc, T> = SeededOption<VecU<'alloc, T>>;
pub type LockstepNU<'alloc, T> = SeededOption<VecNU<'alloc, T>>;