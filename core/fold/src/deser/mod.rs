use serde::ser::SerializeSeq;
use serde_seeded::DeserializeSeeded;

use crate::collections::*;
use crate::common::Vec;

mod shim;

struct ExtendVec<'alloc, 'a, T: 'a>(&'a mut Vec<'alloc, T>, &'alloc bumpalo::Bump);

impl<'de, 'alloc, 'a, T> serde::de::DeserializeSeed<'de> for ExtendVec<'alloc, 'a, T>
where
    T: DeserializeSeeded<'de, Seed<'alloc>>,
{
    type Value = ();

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::de::Deserializer<'de>,
    {
        struct ExtendVecVisitor<'alloc, 'a, T: 'a>(&'a mut Vec<'alloc, T>, &'alloc bumpalo::Bump);

        impl<'de, 'alloc, 'a, T> serde::de::Visitor<'de> for ExtendVecVisitor<'alloc, 'a, T>
        where
            T: DeserializeSeeded<'de, Seed<'alloc>>,
        {
            type Value = ();

            fn expecting(&self, formatter: &mut alloc::fmt::Formatter) -> alloc::fmt::Result {
                write!(formatter, "an array of T")
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: serde::de::SeqAccess<'de>,
            {
                // Decrease the number of reallocations if there are many elements
                if let Some(size_hint) = seq.size_hint() {
                    self.0.reserve(size_hint);
                }

                // Visit each element in the inner array and push it onto
                // the existing vector.
                let seed = Seed(self.1);
                while let Some(elem) = seq.next_element_seed(shim::Seed::new(&seed))? {
                    self.0.push(elem);
                }

                Ok(())
            }
        }

        deserializer.deserialize_seq(ExtendVecVisitor(self.0, self.1))
    }
}

pub struct VecNUVisitor<'alloc, T> {
    allocator: &'alloc bumpalo::Bump,
    _marker: core::marker::PhantomData<T>,
}

impl<'alloc, T> VecNUVisitor<'alloc, T> {
    pub const fn new(allocator: &'alloc bumpalo::Bump) -> Self {
        Self {
            allocator,
            _marker: core::marker::PhantomData,
        }
    }
}

impl<'de, 'alloc, T> serde::de::Visitor<'de> for VecNUVisitor<'alloc, T>
where
    T: DeserializeSeeded<'de, Seed<'alloc>> + 'alloc,
{
    type Value = VecNU<'alloc, T>;

    fn expecting(&self, formatter: &mut alloc::fmt::Formatter) -> alloc::fmt::Result {
        write!(formatter, "an array of arrays")
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: serde::de::SeqAccess<'de>,
    {
        let size_hint = seq.size_hint().unwrap_or(64);

        let mut indices = Vec::with_capacity_in(size_hint, &self.allocator);
        let mut backing = Vec::with_capacity_in(size_hint * 4, &self.allocator);

        // Each iteration through this loop is one inner array.
        loop {
            let previous_index = backing.len();
            let res = seq.next_element_seed(ExtendVec(&mut backing, &self.allocator))?;
            let length = backing.len() - previous_index;
            indices.push(VecNURange {
                idx: u32::try_from(previous_index).unwrap(),
                count: u32::try_from(length).unwrap(),
            });
            if let None = res {
                break;
            }
        }

        Ok(VecNU { backing, indices })
    }
}

pub struct VecVisitor<'alloc, T> {
    allocator: &'alloc bumpalo::Bump,
    _marker: core::marker::PhantomData<T>,
}

impl<'alloc, T> VecVisitor<'alloc, T> {
    pub const fn new(allocator: &'alloc bumpalo::Bump) -> Self {
        Self {
            allocator,
            _marker: core::marker::PhantomData,
        }
    }
}

impl<'de, 'alloc, T> serde::de::Visitor<'de> for VecVisitor<'alloc, T>
where
    T: DeserializeSeeded<'de, Seed<'alloc>> + 'alloc,
{
    type Value = Vec<'alloc, T>;

    fn expecting(&self, formatter: &mut alloc::fmt::Formatter) -> alloc::fmt::Result {
        write!(formatter, "an array of items")
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: serde::de::SeqAccess<'de>,
    {
        let size_hint = seq.size_hint().unwrap_or(64);
        let mut backing = Vec::with_capacity_in(size_hint, &self.allocator);

        // Each iteration through this loop is one inner array.
        while let Some(_) = seq.next_element_seed(ExtendVec(&mut backing, self.allocator))? {}

        Ok(backing)
    }
}

pub struct StringVisitor<'alloc> {
    allocator: &'alloc bumpalo::Bump,
}

impl<'alloc> StringVisitor<'alloc> {
    pub const fn new(allocator: &'alloc bumpalo::Bump) -> Self {
        Self { allocator }
    }
}

impl<'de, 'alloc> serde::de::Visitor<'de> for StringVisitor<'alloc> {
    type Value = bumpalo::collections::String<'alloc>;

    fn expecting(&self, formatter: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(formatter, "a string")
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(bumpalo::collections::String::from_str_in(
            v,
            &self.allocator,
        ))
    }
}

#[derive(Copy, Clone)]
pub struct Seed<'seed>(&'seed bumpalo::Bump);

impl<'seed> Seed<'seed> {
    pub const fn from_bump(inner: &'seed bumpalo::Bump) -> Self {
        Self(inner)
    }
}

impl<'seed, 'de, T> DeserializeSeeded<'de, Seed<'seed>> for VecU<'seed, T>
where
    T: DeserializeSeeded<'de, Seed<'seed>>,
{
    fn deserialize_seeded<D>(seed: &Seed<'seed>, deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer
            .deserialize_seq(VecVisitor::new(seed.0))
            .map(|v| Self(v))
    }
}

impl<'seed, 'de> DeserializeSeeded<'de, Seed<'seed>> for crate::collections::String<'seed> {
    fn deserialize_seeded<D>(seed: &Seed<'seed>, deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer
            .deserialize_str(StringVisitor::new(seed.0))
            .map(|v| Self(v))
    }
}

impl<'seed, 'de, T> DeserializeSeeded<'de, Seed<'seed>> for VecNU<'seed, T>
where
    T: DeserializeSeeded<'de, Seed<'seed>>,
{
    fn deserialize_seeded<D>(seed: &Seed<'seed>, deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_seq(VecNUVisitor::new(seed.0))
    }
}

struct OptionVisitor<'seed, Q: ?Sized, T>(&'seed Q, core::marker::PhantomData<T>);

impl<'de, Q, T> serde::de::Visitor<'de> for OptionVisitor<'_, Q, T>
where
    Q: ?Sized,
    T: DeserializeSeeded<'de, Q>,
{
    type Value = core::option::Option<T>;

    fn expecting(&self, formatter: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(formatter, "an optional value")
    }

    fn visit_none<E>(self) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(None)
    }

    fn visit_some<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        T::deserialize_seeded(self.0, deserializer).map(Some)
    }
}

impl<'seed, 'de, T> DeserializeSeeded<'de, Seed<'seed>> for crate::collections::SeededOption<T>
where
    T: DeserializeSeeded<'de, Seed<'seed>>,
{
    fn deserialize_seeded<D>(seed: &Seed<'seed>, deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer
            .deserialize_option(OptionVisitor(seed, core::marker::PhantomData))
            .map(|inside| Self(inside))
    }
}

/*
impl<'seed, 'de, T> DeserializeSeeded<'de, Seed<'seed>> for Lockstep<'seed, T>
where
    T: serde::de::Deserialize<'de>,
{
    fn deserialize_seeded<D>(seed: &Seed<'seed>, deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_option(OptionVisitor(seed, core::marker::PhantomData)).map(|inside| Self(inside))
    }
}

impl<'seed, 'de, T> DeserializeSeeded<'de, Seed<'seed>> for LockstepNU<'seed, T>
where
    T: serde::de::Deserialize<'de>,
{
    fn deserialize_seeded<D>(seed: &Seed<'seed>, deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_option(OptionVisitor(seed, core::marker::PhantomData)).map(|inside| Self(inside))
    }
}*/

static_assertions::assert_impl_all!(Lockstep<'static, u8>: serde_seeded::DeserializeSeeded<'static, crate::deser::Seed<'static>>);
static_assertions::assert_impl_all!(LockstepNU<'static, u8>: serde_seeded::DeserializeSeeded<'static, crate::deser::Seed<'static>>);

impl<'alloc, T> serde::Serialize for crate::collections::VecNU<'alloc, T>
where
    T: serde::Serialize,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut outer_seq = serializer.serialize_seq(Some(self.indices.len()))?;
        for indices in self {
            outer_seq.serialize_element(indices)?;
        }
        outer_seq.end()
    }
}

mod check {
    use super::*;

    #[derive(DeserializeSeeded)]
    #[seeded(de(seed(Seed<'bump>)))]
    struct Test<'bump>(VecU<'bump, u8>);
    static_assertions::assert_impl_all!(Test<'static>: serde_seeded::DeserializeSeeded<'static, crate::deser::Seed<'static>>);
}
