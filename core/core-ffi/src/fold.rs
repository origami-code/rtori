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

mod internal {
    type FF<'a> = fold::File<'a>;
    use self_cell::self_cell;

    self_cell!(
        pub(crate) struct FoldFileBump {
            owner: fold::collections::bumpalo::Bump,

            #[covariant]
            dependent: FF,
        }

        impl {Debug}
    );
}
use internal::*;

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
        pub fn format(&self, out: &mut DiplomatWrite) {
            self.format_common(out).unwrap()
        }
    }

    #[diplomat::opaque]
    #[diplomat::rust_link(fold::File, Struct)]
    #[derive(Debug)]
    pub struct FoldFile<'ctx> {
        pub(crate) inner: super::FoldFileBump,
        _marker: core::marker::PhantomData<&'ctx super::FoldFileBump>
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
        pub fn format(&self, out: &mut DiplomatWrite) {
            self.format_common(out).unwrap()
        }
    }

    impl<'ctx> Clone for FoldFile<'ctx> {
        fn clone(&self) -> FoldFile<'ctx> {
            let bytes = postcard::to_allocvec(&self.inner.borrow_dependent()).unwrap();
            let mut deserializer = postcard::Deserializer::from_bytes(&bytes);

            let bump = fold::collections::bumpalo::Bump::new();
            let inner = super::FoldFileBump::new(bump, |b| {
                let seed = fold::Seed::from_bump(&b);
                use fold::collections::serde_seeded::DeserializeSeeded;
           
                let parsed = fold::File::deserialize_seeded(&seed, &mut deserializer).expect("this is a roundtrip");
                parsed
            });

            Self {
                inner,
                _marker: core::marker::PhantomData
            }
        }
    }

    impl<'ctx> FoldFile<'ctx> {
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

            let bump = fold::collections::bumpalo::Bump::new();
            let inner = super::FoldFileBump::try_new(bump, |b| {
                let mut deserializer = serde_json::Deserializer::from_slice(bytes);
                let seed = fold::Seed::from_bump(&b);

                use fold::collections::serde_seeded::DeserializeSeeded;
                fold::File::deserialize_seeded(&seed, &mut deserializer).map_err(|inner| FoldFileParseError {
                    status: FoldFileParseErrorKind::Error,
                    error: DiplomatOption::from(Some(JSONParseError::from(inner))),
                })
            })?;

            let ff = Self {
                inner,
                _marker: core::marker::PhantomData
            };

            Ok(Box::new_in(
                ff,
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

            let bump = fold::collections::bumpalo::Bump::new();
            let inner = super::FoldFileBump::try_new(bump, |b| {
                let mut deserializer = serde_json::Deserializer::from_str(str);
                let seed = fold::Seed::from_bump(&b);

                use fold::collections::serde_seeded::DeserializeSeeded;
                fold::File::deserialize_seeded(&seed, &mut deserializer).map_err(|inner| FoldFileParseError {
                    status: FoldFileParseErrorKind::Error,
                    error: DiplomatOption::from(Some(JSONParseError::from(inner))),
                })
            })?;

            let ff = Self {
                inner,
                _marker: core::marker::PhantomData
            };

            Ok(Box::new_in(
                ff,
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

            let metadata =& self
                .inner
                .borrow_dependent()
                .file_metadata;


            let result_str = match field {
                FoldMetadataQuery::Creator => metadata.creator.as_ref(),
                FoldMetadataQuery::Author => metadata.author.as_ref(),
                _ => unreachable!(),
            }.map(|x| x.as_str());

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

        pub fn frame<'a>(&'a self, index: u16) -> Option<Box<FoldFrame<'a>>> {
            self.inner.borrow_dependent().frame(index)
                .map(|inner| Box::new(FoldFrame {inner}))
        }
    }

    #[repr(C)]
    #[diplomat::rust_link(fold::FrameRef, Struct)]
    #[derive(Debug, PartialEq)]
    pub enum FoldFrameKind {
        /// The key frame is the default one in a fold file
        Key,
        /// A frame that is self-contained, even if it might be referring to another one,
        /// it does not inherit from it
        NonInheriting,
        /// A frame that "patches" another one with changes
        Inheriting
    }
    
    /// A reference to a fold frame
    #[diplomat::opaque]
    #[diplomat::rust_link(fold::FrameRef, Struct)]
    #[derive(Debug, Clone, Copy)]
    pub struct FoldFrame<'fold> {
        inner: fold::FrameRef<'fold>
    }

    use fold::{Frame, FrameVertices, FrameEdges, FrameFaces};

    impl<'f> FoldFrame<'f> {
        pub fn kind(&self) -> FoldFrameKind {
            match self.inner {
                fold::FrameRef::Key(_) => FoldFrameKind::Key,
                fold::FrameRef::NonInheriting{..} => FoldFrameKind::NonInheriting,
                fold::FrameRef::Inheriting(_) => FoldFrameKind::Inheriting
            }
        }

        pub fn vertices_count(&self) -> u32 {
            (&self.inner).vertices().count() as u32
        }

        pub fn edges_count(&self) -> u32 {
            (&self.inner).edges().count() as u32
        }

        pub fn faces_count(&self) -> u32 {
            (&self.inner).faces().count() as u32
        }

        pub fn iterate_vertices(&self) -> Box<VerticesIterator<'f>> {
            Box::new(VerticesIterator { inner: *self, cursor: 0 })
        }

        pub fn iterate_edges(&self) -> Box<EdgesIterator<'f>> {
            Box::new(EdgesIterator { inner: *self, cursor: 0 })
        }

        pub fn iterate_faces(&self) -> Box<FacesIterator<'f>> {
            Box::new(FacesIterator { inner: *self, cursor: 0 })
        }

        /* access & copy */
        
        #[diplomat::attr(auto, getter = "vertices_coords")]
        pub fn vertices_coords(&self) -> &'f [f32] {
            todo!()
        }

        pub fn vertices_coords_copy(&self, dst: &mut[f32], offset: u32) -> u32 {
            todo!()
        }

        #[diplomat::attr(auto, getter = "vertices_edges")]
        pub fn vertices_edges(&self) -> &'f [u32] {
            todo!()
        }

        pub fn vertices_edges_copy(&self, dst: &mut[u32], offset: u32) -> u32 {
            todo!()
        }

        #[diplomat::attr(auto, getter = "vertices_faces")]
        pub fn vertices_faces(&self) -> &'f [u32] {
            todo!()
        }

        pub fn vertices_faces_copy(&self, dst: &mut[u32], offset: u32) -> u32 {
            todo!()
        }

        #[diplomat::attr(auto, getter = "edges_vertices")]
        pub fn edges_vertices(&self) -> &'f [u32] {
            todo!()
        }

        pub fn edges_vertices_copy(&self, dst: &mut[u32], offset: u32) -> u32 {
            todo!()
        }

        #[diplomat::attr(auto, getter = "edges_faces")]
        pub fn edges_faces(&self) -> &'f [u32] {
            todo!()
        }

        pub fn edges_faces_copy(&self, dst: &mut[u32], offset: u32) -> u32 {
            todo!()
        }
    }

    /// This is a cursor into the flattened values given by the raw_ methods on a frame
    #[repr(C)]
    pub struct RawSpan {
        pub start: u32,
        pub length: u32
    }

    #[diplomat::opaque]
    pub struct VerticesIterator<'frame> {
        inner: FoldFrame<'frame>,
        cursor: fold::VertexIndex
    }

    impl<'f> VerticesIterator<'f> {
        #[diplomat::attr(auto, iterator)]
        pub fn next(&mut self) -> Option<u32> {
            let next_cursor = self.cursor + 1;
            if next_cursor >= self.inner.vertices_count() {
                None
            } else {
                self.cursor = next_cursor;
                Some(next_cursor)
            }
        }

        /// Writes the vertices coordinates corresponding to the current vertex index (from `vertices_coords`)
        /// Might be 2D or 3D
        pub fn coords(&self, dst: &mut [f32]) -> u32 {
            todo!()
        }

        /// Writes the vertex indices corresponding to the neighbours of the current vertex index (from `vertices_vertices`)
        pub fn vertices(&self, dst: &mut [u32]) -> u32 {
            todo!()
        }

        pub fn vertices_bounds(&self) -> Option<RawSpan> {
            todo!()
        }

        /// Writes the edge indices corresponding to the current vertex index (from `vertices_edges`)
        pub fn edges(&self, dst: &mut [u32]) -> u32 {
            todo!()
        }

        pub fn edges_bounds(&self) -> Option<RawSpan> {
            todo!()
        }

        /// Writes the face indices corresponding to the current vertex index (from `vertices_faces`)
        /// Null values will be written as U32::MAX
        pub fn faces(&self, dst: &mut [u32]) -> u32 {
            todo!()
        }

        pub fn faces_bounds(&self) -> Option<RawSpan> {
            todo!()
        }
    }

    #[diplomat::enum_convert(fold::EdgeAssignment)]
    #[repr(C)]
    pub enum EdgeAssignment {
        /// Border/boundary edge
        B,

        /// Montain Crease
        M,

        /// Valley Crease
        V,

        /// Unassigned/Unknown crease
        U,

        /// Cut/slit edge
        C,

        /// Join edge
        J,

        /// Facet
        F,
    }

    #[repr(C)]
    pub struct Edge {
        pub from: u32,
        pub to: u32
    }

    #[diplomat::opaque]
    pub struct EdgesIterator<'frame> {
        inner: FoldFrame<'frame>,
        cursor: fold::EdgeIndex
    }

    impl<'f> EdgesIterator<'f> {
        #[diplomat::attr(auto, iterator)]
        pub fn next(&mut self) -> Option<u32> {
            let next_cursor = self.cursor + 1;
            if next_cursor >= self.inner.edges_count() {
                None
            } else {
                self.cursor = next_cursor;
                Some(next_cursor)
            }
        }

        /// Writes the vertex indices corresponding to the current edge index (from `edges_vertices`)
        /// It's always two per edge
        pub fn vertices(&self) -> Option<Edge> {
            todo!()
        }

        /// Writes the face indices corresponding to the neighbours of the current edge index (from `edges_faces`)
        /// If null, it will write a U32::MAX
        pub fn faces(&self, dst: &mut [u32]) -> u32 {
            todo!()
        }

        pub fn faces_bounds(&self) -> Option<RawSpan> {
            todo!()
        }

        /// Writes the edge assignment corresponding to the current edge index (from `edges_assignment`)
        pub fn assignment(&self) -> Option<EdgeAssignment> {
            todo!()
        }

        /// From `edges_foldAngle`
        pub fn fold_angle(&self) -> Option<f32> {
            todo!()
        }

        /// From `edges_length`
        pub fn length(&self) -> Option<f32> {
            todo!()
        }
    }

    #[diplomat::opaque]
    pub struct FacesIterator<'frame> {
        inner: FoldFrame<'frame>,
        cursor: fold::FaceIndex
    }

    impl<'f> FacesIterator<'f> {
        #[diplomat::attr(auto, iterator)]
        pub fn next(&mut self) -> Option<u32> {
            let next_cursor = self.cursor + 1;
            if next_cursor >= self.inner.faces_count() {
                None
            } else {
                self.cursor = next_cursor;
                Some(next_cursor)
            }
        }

        /// Writes the vertex indices corresponding to the current face (from `faces_vertices`)
        pub fn vertices(&self, dst: &mut [u32]) -> u32 {
            todo!()
        }

        pub fn vertices_bounds(&self) -> Option<RawSpan> {
            todo!()
        }

        /// Writes the edge indices corresponding to the current face (from `edges_vertices`),
        /// in counterclockwise order
        pub fn edges(&self, dst: &mut [u32]) -> u32 {
            todo!()
        }

        pub fn edges_bounds(&self) -> Option<RawSpan> {
            todo!()
        }

        /// Writes the face indices corresponding to the neighbours of the current face (from `faces_vertices`)
        /// If null, it will write a U32::MAX
        pub fn faces(&self, dst: &mut [u32]) -> u32 {
            todo!()
        }

        pub fn faces_bounds(&self) -> Option<RawSpan> {
            todo!()
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


    pub enum FoldMetadataQuery {
        /// Implies the use of [`query_metadata_string`]
        Creator,
        /// Implies the use of [`string_output`] as data parameter
        Author,
        /// Implies the use of [`u16_output`] as data parameter
        FrameCount,
    }
}
