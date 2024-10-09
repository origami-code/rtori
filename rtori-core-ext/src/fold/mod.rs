/*
pub extern "C" fn rtori_sim_configure(
    sim
)

*/

// TODO: vectored reader

mod transformed;

pub struct FoldFile<'alloc> {
    pub(crate) ctx: crate::Arc<'alloc, crate::Context<'alloc>>,
    pub(crate) parsed: fold::File,
}

#[repr(u8)]
pub enum FoldOperationStatus {
    Success = 0x00,
    ParseError,
    UnknownField,
    ErrorNoSuchFrame,
}

#[repr(u8)]
pub enum FoldParseStatus {
    Success = 0x00,
    /// No data given
    Empty,
    Error,
}

#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub enum JsonParseErrorCategory {
    /// failure to read or write bytes on an I/O stream
    IO,
    ///  input that is not syntactically valid JSON
    Syntax,
    /// input data that is semantically incorrect
    Data,
    /// unexpected end of the input data
    Eof,
}

impl From<serde_json::error::Category> for JsonParseErrorCategory {
    fn from(value: serde_json::error::Category) -> Self {
        match value {
            serde_json::error::Category::Io => Self::IO,
            serde_json::error::Category::Syntax => Self::Syntax,
            serde_json::error::Category::Data => Self::Data,
            serde_json::error::Category::Eof => Self::Eof,
        }
    }
}

#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct JsonParseError {
    pub line: usize,
    pub column: usize,
    pub category: JsonParseErrorCategory,
}

impl From<serde_json::Error> for JsonParseError {
    fn from(value: serde_json::Error) -> Self {
        Self {
            line: value.line(),
            column: value.column(),
            category: JsonParseErrorCategory::from(value.classify()),
        }
    }
}

#[repr(C)]
pub union FoldParsePayload<'alloc> {
    pub file: *const FoldFile<'alloc>,
    pub error: JsonParseError,
    pub nothing: PhantomData<u8>,
}

#[repr(C)]
pub struct FoldParseResult<'alloc> {
    pub status: FoldParseStatus,
    pub payload: FoldParsePayload<'alloc>,
}

// TODO: output errors
/// cbindgen:prefix=RTORI_SLICE_RO(2, 3)
/// cbindgen:ptrs-as-arrays=[[fold_str; ]]
#[no_mangle]
pub unsafe extern "C" fn rtori_fold_parse<'alloc>(
    ctx: *const crate::Context<'alloc>,
    fold_str: *const u8,
    fold_str_len: usize,
) -> FoldParseResult<'alloc> {
    if fold_str_len == 0 {
        return FoldParseResult {
            status: FoldParseStatus::Empty,
            payload: FoldParsePayload {
                nothing: PhantomData,
            },
        };
    }

    let ctx = {
        let ctx = unsafe { crate::Arc::from_raw_in(ctx, (&*ctx).allocator) };
        let ctx_clone = ctx.clone();
        core::mem::forget(ctx);
        ctx_clone
    };

    let fold_source = unsafe { core::slice::from_raw_parts(fold_str, fold_str_len) };
    let parsed = match serde_json::from_slice::<fold::File>(fold_source) {
        Ok(parsed) => parsed,
        Err(e) => {
            return FoldParseResult {
                status: FoldParseStatus::Error,
                payload: FoldParsePayload {
                    error: JsonParseError::from(e),
                },
            }
        }
    };

    let allocator = ctx.allocator;
    let wrapper = FoldFile { ctx, parsed };

    let output = crate::Arc::new_in(wrapper, allocator);
    let output_raw = crate::Arc::into_raw(output);

    FoldParseResult {
        status: FoldParseStatus::Success,
        payload: FoldParsePayload {
            file: output_raw as *mut _,
        },
    }
}

#[repr(C)]
pub struct FoldEncodeResult {
    pub ok: core::ffi::c_int,
    pub written: usize,
}

/// cbindgen:prefix=RTORI_SLICE_WO(2, 3)
/// cbindgen:ptrs-as-arrays=[[fold_str; ]]
#[no_mangle]
pub unsafe extern "C" fn rtori_fold_encode<'alloc>(
    fold: *const FoldFile<'alloc>,
    output: *mut u8,
    output_size: usize,
) -> FoldEncodeResult {
    // TODO
    FoldEncodeResult { ok: 0, written: 0 }
}

/// Drops a fold object. After dropping, the pointer is freed and it should not be used anymore.
#[no_mangle]
pub unsafe extern "C" fn rtori_fold_deinit<'alloc>(fold: *const FoldFile<'alloc>) {
    let _fold = unsafe { crate::Arc::from_raw_in(fold, (&*fold).ctx.allocator) };
    // let it drop naturally
}

#[derive(Copy, Clone)]
#[repr(C)]
pub enum FoldMetadataQuery {
    /// Implies the use of a [`string_output`] as data parameter
    Creator,
    /// Implies the use of a [`string_output`] as data parameter
    Author,
    /// Implies the use of a [`u16_output`] as data parameter
    FrameCount,
}

#[no_mangle]
pub unsafe extern "C" fn rtori_fold_query_metadata<'alloc>(
    fold: *const FoldFile<'alloc>,
    query: FoldMetadataQuery,
    mut output: core::ptr::NonNull<crate::QueryOutput>,
) -> FoldOperationStatus {
    let fold = unsafe { &*fold };

    match query {
        field @ (FoldMetadataQuery::Creator | FoldMetadataQuery::Author) => {
            let source = fold
                .parsed
                .file_metadata
                .as_ref()
                .and_then(|metadata| match field {
                    FoldMetadataQuery::Creator => metadata.creator.as_ref(),
                    FoldMetadataQuery::Author => metadata.author.as_ref(),
                    _ => unreachable!(),
                })
                .map(|x| x.as_str());

            // SAFETY: output must be pointing to valid memory
            unsafe { output.as_mut().copy_str(source) };
            FoldOperationStatus::Success
        }
        FoldMetadataQuery::FrameCount => {
            // SAFETY: output must be pointing to valid memory
            unsafe { output.as_mut().u16_output.as_uninit_mut() }.write(fold.parsed.frame_count());
            FoldOperationStatus::Success
        }
    }
}

#[derive(Copy, Clone)]
#[repr(C)]
pub enum FoldFrameQuery {
    /// Implies the use of a [`QueryOutput::string_output`] as data parameter
    Title,
    /// Implies the use of a [`QueryOutput::string_output`] as data parameter
    Description,
    /// Implies the use of a [`QueryOutput::u32_output`] as data parameter
    VerticesCount,
    /// Implies the use of a [`QueryOutput::u32_output`] as data parameter
    EdgesCount,
    /// Implies the use of a [`QueryOutput::u32_output`] as data parameter
    FacesCount,
    /// Implies the use of a [`QueryOutput::u32_output`] as data parameter
    UVCount,
    /// Outputs the frame's `rtori:uvs`
    /// Implies the use of a [`QueryOutput::vec2f_array_output`] as data parameter
    UVs,
}

#[no_mangle]
pub unsafe extern "C" fn rtori_fold_query_frame<'alloc>(
    fold: *const FoldFile<'alloc>,
    frame_index: u16,
    query: FoldFrameQuery,
    mut output: core::ptr::NonNull<crate::QueryOutput>,
) -> FoldOperationStatus {
    let fold = unsafe { &*fold };
    let frame = match fold.parsed.frame(frame_index) {
        Some(frame) => frame,
        None => return FoldOperationStatus::ErrorNoSuchFrame,
    };

    let frame = frame.get();

    match query {
        field @ (FoldFrameQuery::Title | FoldFrameQuery::Description) => {
            let source = match field {
                FoldFrameQuery::Title => frame.metadata.title.as_ref(),
                FoldFrameQuery::Description => frame.metadata.description.as_ref(),
                _ => unreachable!(),
            }
            .map(|x| x.as_str());

            // SAFETY: output must be pointing to valid memory
            unsafe { output.as_mut().copy_str(source) };
            FoldOperationStatus::Success
        }
        field @ (FoldFrameQuery::VerticesCount
        | FoldFrameQuery::EdgesCount
        | FoldFrameQuery::FacesCount
        | FoldFrameQuery::UVCount) => {
            let count = u32::try_from(match field {
                FoldFrameQuery::VerticesCount => frame.vertices.count(),
                FoldFrameQuery::EdgesCount => frame.edges.count(),
                FoldFrameQuery::FacesCount => frame.faces.count(),
                FoldFrameQuery::UVCount => frame.uvs.as_ref().map(|v| v.len()).unwrap_or(0),
                _ => unreachable!(),
            })
            .unwrap();

            // SAFETY: output must be pointing to valid memory
            unsafe { output.as_mut().u32_output.as_uninit_mut() }.write(count);
            FoldOperationStatus::Success
        }
        FoldFrameQuery::UVs => {
            let source = frame.uvs.as_ref().map(|v| v.as_slice());
            unsafe { output.as_mut().copy_vec2f(source) };
            FoldOperationStatus::Success
        }
    }
}

//pub unsafe extern "C" fn rtori_fold_copy

// TODO: drop
// TODO: queries

use std::marker::PhantomData;

// TODO: output errors
pub use transformed::*;
