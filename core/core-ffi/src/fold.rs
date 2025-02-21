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
#[diplomat::attr(auto, namespace = "rtori")] // todo: ::fold when https://github.com/rust-diplomat/diplomat/issues/591
pub mod ffi {
    use crate::context::ffi as context;

    use diplomat_runtime::DiplomatByte;
    use diplomat_runtime::DiplomatWrite;

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
        pub(crate) inner: fold::File, //<'ctx>
        _marker: std::marker::PhantomData<&'ctx fold::File>,
    }

    #[derive(Debug)]
    pub enum FoldFileParseErrorKind {
        Empty,
        Error,
    }

    #[derive(Debug)]
    #[repr(C)]
    pub struct FoldFileParseError {
        pub status: FoldFileParseErrorKind,
        pub error: DiplomatOption<JSONParseError>,
    }

    impl<'ctx> FoldFile<'ctx> {
        // Disabled on dart, not sure why but I get a "custom handling" unreachable! invoke
        #[diplomat::attr(dart, disable)]
        pub fn parse_bytes(
            ctx: &context::Context<'ctx>,
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
            ctx: &context::Context<'ctx>,
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
