use crate::common::Vec;
use crate::handful::*;

struct ExtendVec<'alloc, 'a, T: 'a>(&'a mut Vec<'alloc, T>);

impl<'de, 'alloc, 'a, T> serde::de::DeserializeSeed<'de> for ExtendVec<'alloc, 'a, T>
where
    T: serde::Deserialize<'de>,
{
    type Value = ();

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::de::Deserializer<'de>,
    {
        struct ExtendVecVisitor<'alloc, 'a, T: 'a>(&'a mut Vec<'alloc, T>);

        impl<'de, 'alloc, 'a, T> serde::de::Visitor<'de> for ExtendVecVisitor<'alloc, 'a, T>
        where
            T: serde::de::Deserialize<'de>,
        {
            type Value = ();

            fn expecting(&self, formatter: &mut alloc::fmt::Formatter) -> alloc::fmt::Result {
                write!(formatter, "an array of integers")
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
                while let Some(elem) = seq.next_element()? {
                    self.0.push(elem);
                }

                Ok(())
            }
        }

        deserializer.deserialize_seq(ExtendVecVisitor(self.0))
    }
}

// Visitor implementation that will walk the outer array of the JSON input.
struct FlattenedVecVisitor<'alloc, T> {
    allocator: &'alloc bumpalo::Bump,
    _marker: core::marker::PhantomData<T>,
}

impl<'de, 'alloc, T> serde::de::Visitor<'de> for FlattenedVecVisitor<'alloc, T>
where
    T: serde::de::Deserialize<'de> + 'alloc,
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
            let res = seq.next_element_seed(ExtendVec(&mut backing))?;
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
