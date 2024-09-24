use core::{marker::PhantomData, ops::{Deref, DerefMut}};

pub struct Proxy<Collection, InnerType> {
    collection: Collection,
    _inner_type: PhantomData<InnerType>
}

impl<Collection, InnerType> Proxy<Collection, InnerType> {
    pub const fn new(collection: Collection) -> Self {
        Self {
            collection,
            _inner_type: PhantomData
        }
    }
}

impl<Collection, InnerType> Deref for Proxy<Collection, InnerType>
where
    Collection: Deref<Target=[u8]>,
    InnerType: bytemuck::AnyBitPattern
{
    type Target = [InnerType];

    fn deref(&self) -> &Self::Target {
        bytemuck::cast_slice(&self.collection)
    }
}

impl<Collection, InnerType> DerefMut for Proxy<Collection, InnerType>
where
    Collection: DerefMut<Target=[u8]>,
    InnerType: bytemuck::AnyBitPattern + bytemuck::NoUninit
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        bytemuck::cast_slice_mut(&mut self.collection)
    }
}
