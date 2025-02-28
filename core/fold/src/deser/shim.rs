use serde::de::DeserializeSeed;
use serde_seeded::DeserializeSeeded;
/// Seed deserializing any `T` implementing `DeserializeSeeded<Q>`.
///
/// This type implements [`DeserializeSeed`] when `T` implements
/// [`DeserializeSeeded<Q>`].
pub struct Seed<'a, Q: ?Sized, T> {
    seed: &'a Q,
    t: core::marker::PhantomData<T>,
}

impl<'a, Q: ?Sized, T> Seed<'a, Q, T> {
    /// Creates a new deserializing seed.
    pub fn new(seed: &'a Q) -> Self {
        Self {
            seed,
            t: core::marker::PhantomData,
        }
    }
}

impl<Q: ?Sized, T> Clone for Seed<'_, Q, T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<Q: ?Sized, T> Copy for Seed<'_, Q, T> {}

impl<'de, Q, T> DeserializeSeed<'de> for Seed<'_, Q, T>
where
    Q: ?Sized,
    T: DeserializeSeeded<'de, Q>,
{
    type Value = T;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        T::deserialize_seeded(self.seed, deserializer)
    }
}
