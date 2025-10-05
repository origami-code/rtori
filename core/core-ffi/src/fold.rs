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

impl ffi::JSONParseErrorCategory {
    fn format_common<W: core::fmt::Write>(&self, mut f: W) -> core::fmt::Result {
        match self {
            Self::IO => write!(f, "io error"),
            Self::Syntax => write!(f, "invalid JSON syntax"),
            Self::Data => write!(f, "semantically invalid data"),
            Self::Eof => write!(f, "uexpected eof"),
        }
    }
}

impl core::fmt::Display for ffi::JSONParseErrorCategory {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        self.format_common(f)
    }
}

impl ffi::JSONParseError {
    pub fn format_common<W: core::fmt::Write>(&self, mut f: W) -> core::fmt::Result {
        write!(
            f,
            "json parsing error on line {}:{}: {}",
            self.line, self.column, self.category
        )
    }
}

impl core::fmt::Display for ffi::JSONParseError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        self.format_common(f)
    }
}

impl core::error::Error for ffi::JSONParseError {}

impl ffi::FoldFileParseError {
    pub fn format_common<W: core::fmt::Write>(&self, mut f: W) -> core::fmt::Result {
        match self.status {
            ffi::FoldFileParseErrorKind::Empty => write!(f, "empty FOLD source given"),
            ffi::FoldFileParseErrorKind::Error => write!(
                f,
                "error while parsing the fold file: {}",
                self.error.as_ref().unwrap()
            ),
        }
    }
}

impl core::fmt::Display for ffi::FoldFileParseError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        self.format_common(f)
    }
}

impl core::error::Error for ffi::FoldFileParseError {}

#[diplomat::bridge]
#[diplomat::abi_rename = "rtori_{0}"]
#[diplomat::attr(auto, namespace = "rtori")] // todo: ::fold when https://github.com/rust-diplomat/diplomat/issues/591
pub mod ffi {
    use crate::context::ffi as context;

    use diplomat_runtime::DiplomatByte;
    use diplomat_runtime::DiplomatWrite;

    #[derive(Debug)]
    #[repr(C)]
    pub enum JSONParseErrorCategory {
        /// failure to read or write bytes on an I/O stream
        IO,
        /// input that is not syntactically valid JSON
        Syntax,
        /// input data that is semantically incorrect
        Data,
        /// unexpected end of the input data
        Eof,
    }

    impl JSONParseErrorCategory {
        #[diplomat::attr(auto, stringifier)]
        pub fn format(&self, out: &mut DiplomatWrite) {
            self.format_common(out).unwrap()
        }
    }

    #[derive(Debug, Clone)]
    #[repr(C)]
    pub struct JSONParseError {
        pub line: u32,
        pub column: u32,
        pub category: JSONParseErrorCategory,
    }

    impl JSONParseError {
        #[diplomat::attr(auto, stringifier)]
        pub fn format(self, out: &mut DiplomatWrite) {
            self.format_common(out).unwrap()
        }
    }

    #[diplomat::opaque]
    #[derive(Debug, Clone)]
    pub struct FoldFile<'ctx> {
        pub(crate) inner: fold::File, //<'ctx>
        _marker: std::marker::PhantomData<&'ctx fold::File>,
    }

    #[derive(Debug)]
    pub enum FoldFileParseErrorKind {
        /// Parsing failed because the input was empty(ish)
        Empty,
        /// Error while parsing the fold file, meaning a JSON error
        Error,
    }

    #[derive(Debug)]
    #[repr(C)]
    pub struct FoldFileParseError {
        pub status: FoldFileParseErrorKind,
        pub error: DiplomatOption<JSONParseError>,
    }

    impl FoldFileParseError {
        #[diplomat::attr(auto, stringifier)]
        pub fn format(self, out: &mut DiplomatWrite) {
            self.format_common(out).unwrap()
        }
    }

    impl<'ctx> FoldFile<'ctx> {
        #[diplomat::abi_rename = "clone"]
        #[diplomat::attr(*, rename = "clone")]
        pub fn ffi_clone(&self) -> Box<FoldFile<'ctx>, crate::A<'ctx>> {
            Box::new(self.clone())
        }

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

        pub fn query_frame_metadata_u32(&self, frame_index: u16, field: FoldFrameInfoQuery) -> Option<u32> {
            let frame_ref = self.inner.frame(frame_index)?;
            Some(match field {
                FoldFrameInfoQuery::VerticesCount => frame_ref.vertices_count() as u32,
                FoldFrameInfoQuery::EdgeCount => frame_ref.edges_count() as u32,
                FoldFrameInfoQuery::FaceCount => frame_ref.faces_count() as u32
            })
        }
    }

    pub enum FoldFrameInfoQuery {
        VerticesCount,
        EdgeCount,
        FaceCount
    }

    pub enum FoldFrameFloatQuery {
        /// corresponds to vertices_coords
        VerticesCoords,
        /// corresponds to edges_foldAangle
        EdgesFoldAngle,
        /// corresponds to edges_length
        EdgesLength
    }

    pub enum FoldFrameIntQuery {
        /// edges_vertices
        /// arity of two
        EdgesVertices,
    }

    
    #[diplomat::opaque]
    #[derive(Debug, Clone)]
    pub struct FoldFrame<'ctx> {
        pub(crate) inner: Box<FoldFile<'ctx>, crate::A<'ctx>>, //<'ctx>
        pub(crate) frame_index: u16
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
