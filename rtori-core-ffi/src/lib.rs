#![feature(allocator_api)]

extern crate alloc;
use alloc::alloc::Allocator;
use core::ffi::c_void;

#[cfg(feature = "allocator_api")]
mod allocator {
    type RawAlloc =
        unsafe extern "C" fn(ctx: *const c_void, size: usize, alignment: usize) -> *mut c_void;
    type RawDealloc =
        unsafe extern "C" fn(ctx: *const c_void, ptr: *mut c_void, size: usize, alignment: usize);

    /// The two function pointed to must be thread-safe
    #[derive(Debug, Clone, Copy, PartialEq)]
    #[repr(C)]
    pub struct CustomAllocator {
        pub alloc: RawAlloc,
        pub dealloc: RawDealloc,
        pub ctx: *const c_void,
    }

    unsafe impl<'alloc> Allocator for &'alloc CustomAllocator {
        fn allocate(
            &self,
            layout: alloc::Layout,
        ) -> Result<std::ptr::NonNull<[u8]>, alloc::AllocError> {
            if layout.size() == 0 {
                return Err(alloc::AllocError);
            }

            let ptr = unsafe { (self.alloc)(self.ctx, layout.size(), layout.align()) }.cast::<u8>();

            let ptr_nonnull = std::ptr::NonNull::new(ptr).ok_or(alloc::AllocError)?;

            return Ok(std::ptr::NonNull::slice_from_raw_parts(
                ptr_nonnull,
                layout.size(),
            ));
        }

        unsafe fn deallocate(&self, ptr: std::ptr::NonNull<u8>, layout: alloc::Layout) {
            if layout.size() == 0 {
                panic!("Should not happen to deallocate a ZST when we should panic on alloc");
            }

            unsafe {
                (self.dealloc)(
                    self.ctx,
                    ptr.as_ptr().cast::<c_void>(),
                    layout.size(),
                    layout.align(),
                )
            };
        }
    }

    #[derive(Debug, Clone, Copy, PartialEq)]
    pub enum ContextAllocator<'alloc> {
        Custom(&'alloc CustomAllocator),
        Global,
    }

    unsafe impl<'alloc> Allocator for ContextAllocator<'alloc> {
        fn allocate(
            &self,
            layout: alloc::Layout,
        ) -> Result<std::ptr::NonNull<[u8]>, alloc::AllocError> {
            match &self {
                ContextAllocator::Custom(given) => given.allocate(layout),
                ContextAllocator::Global => alloc::Global.allocate(layout),
            }
        }

        unsafe fn deallocate(&self, ptr: std::ptr::NonNull<u8>, layout: alloc::Layout) {
            match &self {
                // SAFETY: as long as the invariants are handled by the C code, this is fine
                ContextAllocator::Custom(given) => unsafe { given.deallocate(ptr, layout) },
                // SAFETY: just a passthrough
                ContextAllocator::Global => unsafe { alloc::Global.deallocate(ptr, layout) },
            }
        }
    }

    /// An extension to allow c-language interface to use a custom allocator
    pub extern "C" fn Context_custom(
        custom: std::ptr::NonNull<crate::CustomAllocator>,
    ) -> *const ffi::Context<'static> {
        let allocator = crate::ContextAllocator::Custom(unsafe { &*(custom.as_ref()) });
        let context = ffi::Context { allocator };

        let boxed = Box::new_in(context, allocator);

        return Box::into_raw(boxed);
    }
}

#[cfg(feature = "allocator_api")]
pub type A<'ctx> = std::alloc::Global;

#[cfg(not(feature = "allocator_api"))]
pub type A<'ctx> = std::alloc::Global;

impl From<serde_json::error::Category> for ffi::JSONParseErrorCategory {
    fn from(value: serde_json::error::Category) -> Self {
        match value {
            serde_json::error::Category::Io => Self::IO,
            serde_json::error::Category::Syntax => Self::Syntax,
            serde_json::error::Category::Data => Self::Data,
            serde_json::error::Category::Eof => Self::Eof,
        }
    }
}

impl From<serde_json::Error> for ffi::JSONParseError {
    fn from(value: serde_json::Error) -> Self {
        Self {
            line: value.line().try_into().unwrap_or(u32::MAX),
            column: value.column().try_into().unwrap_or(u32::MAX),
            category: ffi::JSONParseErrorCategory::from(value.classify()),
        }
    }
}

#[diplomat::bridge]
#[diplomat::abi_rename = "rtori_{0}"]
#[diplomat::attr(auto, namespace = "rtori")]
mod ffi {
    use diplomat_runtime::DiplomatByte;
    use diplomat_runtime::DiplomatWrite;

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

    #[derive(Debug)]
    pub enum JSONParseErrorCategory {
        /// failure to read or write bytes on an I/O stream
        IO,
        ///  input that is not syntactically valid JSON
        Syntax,
        /// input data that is semantically incorrect
        Data,
        /// unexpected end of the input data
        Eof,
    }

    #[derive(Debug)]
    pub struct JSONParseError {
        pub line: u32,
        pub column: u32,
        pub category: JSONParseErrorCategory,
    }

    #[diplomat::opaque]
    #[derive(Debug)]
    pub struct FoldFile<'ctx> {
        inner: fold::File, //<'ctx>
        _marker: std::marker::PhantomData<&'ctx fold::File>,
    }

    #[derive(Debug)]
    pub enum FoldFileParseErrorKind {
        Empty,
        Error,
    }

    #[derive(Debug)]
    pub struct FoldFileParseError {
        pub status: FoldFileParseErrorKind,
        pub error: DiplomatOption<JSONParseError>,
    }

    impl<'ctx> FoldFile<'ctx> {
        pub fn parse_bytes(
            ctx: &Context<'ctx>,
            bytes: &[DiplomatByte],
        ) -> Result<Box<FoldFile<'ctx>, crate::A<'ctx>>, FoldFileParseError> {
            if bytes.len() == 0 {
                return Err(FoldFileParseError {
                    status: FoldFileParseErrorKind::Empty,
                    error: DiplomatOption::from(None),
                });
            }

            let parsed = serde_json::from_slice::<fold::File>(bytes).map_err(|inner| {
                FoldFileParseError {
                    status: FoldFileParseErrorKind::Error,
                    error: DiplomatOption::from(Some(JSONParseError::from(inner))),
                }
            })?;

            Ok(Box::new_in(
                FoldFile {
                    inner: parsed,
                    _marker: std::marker::PhantomData,
                },
                ctx.allocator,
            ))
        }

        pub fn parse_str(
            ctx: &Context<'ctx>,
            str: &str,
        ) -> Result<Box<FoldFile<'ctx>, crate::A<'ctx>>, FoldFileParseError> {
            if str.len() == 0 {
                return Err(FoldFileParseError {
                    status: FoldFileParseErrorKind::Empty,
                    error: DiplomatOption::from(None),
                });
            }

            let parsed =
                serde_json::from_str::<fold::File>(str).map_err(|inner| FoldFileParseError {
                    status: FoldFileParseErrorKind::Error,
                    error: DiplomatOption::from(Some(JSONParseError::from(inner))),
                })?;

            Ok(Box::new_in(
                FoldFile {
                    inner: parsed,
                    _marker: std::marker::PhantomData,
                },
                ctx.allocator,
            ))
        }

        pub fn query_metadata_string(
            &self,
            field: FoldMetadataQuery,
            output: &mut DiplomatWrite,
        ) -> Result<(), ()> {
            if !matches!(
                field,
                FoldMetadataQuery::Creator | FoldMetadataQuery::Author
            ) {
                return Err(());
            }

            let result_str = self
                .inner
                .file_metadata
                .as_ref()
                .and_then(|metadata| match field {
                    FoldMetadataQuery::Creator => metadata.creator.as_ref(),
                    FoldMetadataQuery::Author => metadata.author.as_ref(),
                    _ => unreachable!(),
                })
                .map(|x| x.as_str());

            match result_str {
                Some(s) => {
                    use std::fmt::Write;
                    output.write_str(s).unwrap();
                    Ok(())
                }
                None => Err(()),
            }
        }

        pub fn query_metadata_u16(&self, query: FoldMetadataQuery) -> u16 {
            todo!()
        }
    }

    pub enum FoldMetadataQuery {
        /// Implies the use of [`query_metadata_string`]
        Creator,
        /// Implies the use of [`string_output`] as data parameter
        Author,
        /// Implies the use of [`u16_output`] as data parameter
        FrameCount,
    }
}
