#[diplomat::bridge]
#[diplomat::abi_rename = "rtori_{0}"]
#[diplomat::attr(auto, namespace = "rtori")]
pub mod ffi {
    /// A context is an allocation arena, which can be unallocated at any point.
    /// There may be several in a process.
    #[diplomat::opaque]
    #[derive(Debug)]
    pub struct Context<'alloc> {
        pub(crate) allocator: crate::A<'alloc>,
        _marker: core::marker::PhantomData<&'alloc crate::A<'alloc>>,
    }

    impl Context<'static> {
        #[diplomat::attr(auto, constructor)]
        pub const fn global() -> Box<Self, crate::A<'static>> {
            todo!()
        }
    }
}
