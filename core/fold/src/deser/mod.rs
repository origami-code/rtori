use serde::ser::SerializeSeq;
use serde_seeded::DeserializeSeeded;

use crate::collections::*;

mod shim;

#[macro_export]
macro_rules! assert_deserializable {
    (
        $name:ident,
        $candidate:ty
    ) => {
        // The const block hides the free function from an accessible namespace
        const _: () = {
            // The const fn tests that the given $candidate implements DeserializeSeeded
            #[doc(hidden)]
            #[allow(dead_code)]
            const fn $name<Alloc>()
            where
                Alloc: ::core::alloc::Allocator,
                $candidate:
                    for<'a> ::serde_seeded::DeserializeSeeded<'a, $crate::deser::Seed<Alloc>>,
            {
            }

            let _ = $name::<alloc::alloc::Global>();
        };
    };
}

struct ExtendVec<'a, T: 'a, A: core::alloc::Allocator>(&'a mut alloc::vec::Vec<T, A>, A);

impl<'de, 'a, T, A> serde::de::DeserializeSeed<'de> for ExtendVec<'a, T, A>
where
    T: DeserializeSeeded<'de, Seed<A>>,
    A: core::alloc::Allocator,
{
    type Value = ();

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::de::Deserializer<'de>,
    {
        struct ExtendVecVisitor<'a, T: 'a, A>(&'a mut alloc::vec::Vec<T, A>, A)
        where
            A: core::alloc::Allocator;

        impl<'de, 'a, T, A> serde::de::Visitor<'de> for ExtendVecVisitor<'a, T, A>
        where
            T: DeserializeSeeded<'de, Seed<A>>,
            A: core::alloc::Allocator,
        {
            type Value = ();

            fn expecting(&self, formatter: &mut alloc::fmt::Formatter) -> alloc::fmt::Result {
                write!(formatter, "an array of T")
            }

            fn visit_seq<Access>(self, mut seq: Access) -> Result<Self::Value, Access::Error>
            where
                Access: serde::de::SeqAccess<'de>,
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

pub struct VecNUVisitor<T, A> {
    allocator: A,
    _marker: core::marker::PhantomData<T>,
}

impl<T, A> VecNUVisitor<T, A> {
    pub const fn new(allocator: A) -> Self
    where
        A: core::alloc::Allocator,
    {
        Self {
            allocator,
            _marker: core::marker::PhantomData,
        }
    }
}

impl<'de, T, A> serde::de::Visitor<'de> for VecNUVisitor<T, A>
where
    T: DeserializeSeeded<'de, Seed<A>>,
    A: core::alloc::Allocator + Clone,
{
    type Value = VecNU<T, A>;

    fn expecting(&self, formatter: &mut alloc::fmt::Formatter) -> alloc::fmt::Result {
        write!(formatter, "an array of arrays")
    }

    fn visit_seq<Access>(self, mut seq: Access) -> Result<Self::Value, Access::Error>
    where
        Access: serde::de::SeqAccess<'de>,
    {
        let size_hint = seq.size_hint().unwrap_or(64);

        let mut indices = alloc::vec::Vec::with_capacity_in(size_hint, self.allocator.clone());
        let mut backing = alloc::vec::Vec::with_capacity_in(size_hint * 4, self.allocator.clone());

        // Each iteration through this loop is one inner array.
        loop {
            let previous_index = backing.len();
            let res = seq.next_element_seed(ExtendVec(&mut backing, self.allocator.clone()))?;
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

pub struct VecVisitor<T, A>
where
    A: core::alloc::Allocator,
{
    allocator: A,
    _marker: core::marker::PhantomData<T>,
}

impl<'alloc, T, A> VecVisitor<T, A>
where
    A: core::alloc::Allocator,
{
    pub const fn new(allocator: A) -> Self {
        Self {
            allocator,
            _marker: core::marker::PhantomData,
        }
    }
}

impl<'de, T, A> serde::de::Visitor<'de> for VecVisitor<T, A>
where
    T: DeserializeSeeded<'de, Seed<A>>,
    A: core::alloc::Allocator + Clone,
{
    type Value = alloc::vec::Vec<T, A>;

    fn expecting(&self, formatter: &mut alloc::fmt::Formatter) -> alloc::fmt::Result {
        write!(formatter, "an array of items")
    }

    fn visit_seq<Access>(self, mut seq: Access) -> Result<Self::Value, Access::Error>
    where
        Access: serde::de::SeqAccess<'de>,
    {
        let size_hint = seq.size_hint().unwrap_or(64);
        let mut backing = alloc::vec::Vec::with_capacity_in(size_hint, self.allocator.clone());

        // Each iteration through this loop is one inner array.
        while let Some(_) =
            seq.next_element_seed(ExtendVec(&mut backing, self.allocator.clone()))?
        {}

        Ok(backing)
    }
}

#[derive(Debug)]
pub struct StringVisitor<A> {
    allocator: A,
}

impl<A> StringVisitor<A> {
    pub const fn new(allocator: A) -> Self
    where
        A: core::alloc::Allocator,
    {
        Self { allocator }
    }
}

impl<'de, A> serde::de::Visitor<'de> for StringVisitor<A>
where
    A: core::alloc::Allocator,
{
    type Value = string_alloc::String<A>;

    fn expecting(&self, formatter: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(formatter, "a string")
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(string_alloc::String::from_str_in(v, self.allocator))
    }
}

#[derive(Copy, Clone)]
pub struct Seed<A>(A);

impl<A: core::alloc::Allocator + Clone> Seed<A> {
    pub const fn new(allocator: A) -> Self {
        Self(allocator)
    }
}

impl<'de, T, A> DeserializeSeeded<'de, Seed<A>> for VecU<T, A>
where
    T: DeserializeSeeded<'de, Seed<A>>,
    A: core::alloc::Allocator + Clone,
{
    fn deserialize_seeded<D>(seed: &Seed<A>, deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer
            .deserialize_seq(VecVisitor::new(seed.0.clone()))
            .map(|v| Self(v))
    }
}

impl<'de, Alloc> DeserializeSeeded<'de, Seed<Alloc>> for crate::collections::String<Alloc>
where
    Alloc: core::alloc::Allocator + Clone,
{
    fn deserialize_seeded<D>(seed: &Seed<Alloc>, deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer
            .deserialize_str(StringVisitor::new(seed.0.clone()))
            .map(|v| Self(v))
    }
}

impl<'de, T, A> DeserializeSeeded<'de, Seed<A>> for VecNU<T, A>
where
    T: DeserializeSeeded<'de, Seed<A>>,
    A: core::alloc::Allocator + Clone,
{
    fn deserialize_seeded<D>(seed: &Seed<A>, deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_seq(VecNUVisitor::new(seed.0.clone()))
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

impl<'de, T, A> DeserializeSeeded<'de, Seed<A>> for crate::collections::SeededOption<T>
where
    T: DeserializeSeeded<'de, Seed<A>>,
    A: core::alloc::Allocator,
{
    fn deserialize_seeded<D>(seed: &Seed<A>, deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer
            .deserialize_option(OptionVisitor(seed, core::marker::PhantomData))
            .map(|inside| Self(inside))
    }
}

/*
impl<'seed, 'de, T> DeserializeSeeded<'de, Seed<'seed>> for VecU<T, &'seed bumpalo::Bump>
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

impl<'seed, 'de, T> DeserializeSeeded<'de, Seed<'seed>> for VecNU<'seed, T>
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

assert_deserializable!(assert_deserializable_vec_u, VecU<u8, Alloc>);

impl<T, A> serde::Serialize for crate::collections::VecNU<T, A>
where
    T: serde::Serialize,
    A: core::alloc::Allocator,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut outer_seq = serializer.serialize_seq(Some(self.indices.len()))?;
        for indices in self.as_slice() {
            outer_seq.serialize_element(indices)?;
        }
        outer_seq.end()
    }
}

mod check {
    use super::*;

    #[derive(DeserializeSeeded)]
    #[seeded(de(seed(Seed<Alloc>), override_bounds(Alloc: Clone)))]
    struct Test<Alloc: core::alloc::Allocator>(VecU<u8, Alloc>);

    assert_deserializable!(assert_deserializable_test, Test<Alloc>);
}
