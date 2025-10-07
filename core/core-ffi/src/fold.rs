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
    fn to_str(&self) -> &'static str {
        match self {
            Self::Unknown => "unknown",
            Self::IO => "io error",
            Self::Syntax => "invalid JSON syntax",
            Self::Data => "semantically invalid data",
            Self::Eof => "uexpected eof",
        }
    }

    fn format_common<W: core::fmt::Write>(&self, mut f: W) -> core::fmt::Result {
        write!(f, "{}", self.to_str())
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

fn access_u<Scalar, Composed>(src: &[Composed], offset: u32) -> &[Scalar]
where
    Scalar: bytemuck::AnyBitPattern,
    Composed: bytemuck::NoUninit,
{
    bytemuck::cast_slice(&src[(offset as usize)..])
}

fn copy_u<Scalar, Composed>(
    dst: &mut [Scalar],
    src: &[Composed],
    offset: u32,
) -> Result<u32, ffi::UniformCopyError>
where
    Scalar: bytemuck::AnyBitPattern,
    Composed: bytemuck::NoUninit,
{
    let required_alignment = core::mem::size_of::<Composed>() / core::mem::size_of::<Scalar>();
    if dst.len() % required_alignment != 0 {
        return Err(ffi::UniformCopyError::DestinationAlignmentInvalid);
    }

    let src = access_u(src, offset);
    let copy_len = src.len().min(dst.len());
    dst[..copy_len].copy_from_slice(&src[..copy_len]);

    Ok(copy_len as u32)
}

/// If an indices destination is given, then this copies up to that destination buffer size of indices, using the provided offset,
/// as well as the matching backing data, if the destination backing buffer is provided, limited to its capacity.
///
/// If no indices destination is given, the offset is applied to the backing data and the it is copied up to the capacity of the destination backing buffer.
fn copy_nu<T>(
    backing_src: &[T],
    indices_src: &[ffi::RawSpan],
    backing_dst: &mut [T],
    indices_dst: &mut [ffi::RawSpan],
    offset: u32,
) -> ffi::NUCopyInfo
where
    T: num_traits::Num + Copy,
{
    let indices = &indices_src[(offset as usize)..];
    let has_indices_destination = indices_dst.len() > 0;
    let indices_written = if has_indices_destination {
        let src = indices;
        let len = src.len().min(indices_dst.len());
        indices_dst.copy_from_slice(src);
        Some(len as u32)
    } else {
        None
    };

    let has_backing_destination = backing_dst.len() > 0;
    let backing_written = if has_backing_destination {
        let src = backing_src;

        let (from, end) = if let Some(indices_written) = indices_written {
            let from = indices[offset as usize].start;

            let last_index_written = indices_written.saturating_sub(1);
            let max_backing = &indices[last_index_written as usize];
            let end = (src.len() as u32).min(max_backing.start + max_backing.length);

            (from, end)
        } else {
            let from = offset as u32;
            let end = src.len() as u32;

            (from, end)
        };

        // Ensure we don't go over the backing destination
        let backing_wants_to_write = end - from;
        let len = backing_wants_to_write.min(backing_dst.len() as u32);

        let src = &src[(from as usize)..((from + len) as usize)];
        backing_dst.copy_from_slice(src);
        len as u32
    } else {
        0
    };

    ffi::NUCopyInfo {
        backing_written,
        indices_written: indices_written.unwrap_or(0),
    }
}

#[diplomat::bridge]
#[diplomat::abi_rename = "rtori_{0}"]
#[diplomat::attr(auto, namespace = "rtori")] // todo: ::fold when https://github.com/rust-diplomat/diplomat/issues/591
pub mod ffi {

    use crate::context::ffi as context;

    use diplomat_runtime::DiplomatByte;
    use diplomat_runtime::DiplomatWrite;

    #[derive(Debug, Default)]
    #[repr(C)]
    pub enum JSONParseErrorCategory {
        #[default]
        Unknown,
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
        #[diplomat::attr(not(supports = struct_refs), disable)]
        #[diplomat::attr(auto, stringifier)]
        #[diplomat::attr(*, rename = "format")]
        pub fn format_ref(&self, out: &mut DiplomatWrite) {
            self.format_common(out).unwrap()
        }

        #[diplomat::attr(supports = struct_refs, disable)]
        #[diplomat::attr(auto, stringifier)]
        #[diplomat::attr(*, rename = "format")]
        pub fn format(self, out: &mut DiplomatWrite) {
            self.format_common(out).unwrap()
        }
    }

    #[derive(Debug, Clone, Default)]
    #[repr(C)]
    pub struct JSONParseError {
        pub line: u32,
        pub column: u32,
        pub category: JSONParseErrorCategory,
    }

    impl JSONParseError {
        #[diplomat::attr(not(supports = struct_refs), disable)]
        #[diplomat::attr(auto, stringifier)]
        #[diplomat::attr(*, rename = "format")]
        pub fn format_ref(&self, out: &mut DiplomatWrite) {
            self.format_common(out).unwrap()
        }

        #[diplomat::attr(supports = struct_refs, disable)]
        #[diplomat::attr(auto, stringifier)]
        #[diplomat::attr(*, rename = "format")]
        pub fn format(self, out: &mut DiplomatWrite) {
            self.format_common(out).unwrap()
        }
    }

    #[diplomat::opaque]
    #[diplomat::rust_link(fold::File, Struct)]
    #[derive(Debug)]
    pub struct FoldFile<'ctx> {
        pub(crate) inner: super::FoldFileBump,
        _marker: core::marker::PhantomData<&'ctx super::FoldFileBump>,
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
        #[diplomat::attr(not(supports = struct_refs), disable)]
        #[diplomat::attr(auto, stringifier)]
        #[diplomat::attr(*, rename = "format")]
        pub fn format_ref(&self, out: &mut DiplomatWrite) {
            self.format_common(out).unwrap()
        }

        #[diplomat::attr(supports = struct_refs, disable)]
        #[diplomat::attr(auto, stringifier)]
        #[diplomat::attr(*, rename = "format")]
        pub fn format(self, out: &mut DiplomatWrite) {
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

                let parsed = fold::File::deserialize_seeded(&seed, &mut deserializer)
                    .expect("this is a roundtrip");
                parsed
            });

            Self {
                inner,
                _marker: core::marker::PhantomData,
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
                fold::File::deserialize_seeded(&seed, &mut deserializer).map_err(|inner| {
                    FoldFileParseError {
                        status: FoldFileParseErrorKind::Error,
                        error: DiplomatOption::from(Some(JSONParseError::from(inner))),
                    }
                })
            })?;

            let ff = Self {
                inner,
                _marker: core::marker::PhantomData,
            };

            Ok(Box::new_in(ff, ctx.allocator))
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
                fold::File::deserialize_seeded(&seed, &mut deserializer).map_err(|inner| {
                    FoldFileParseError {
                        status: FoldFileParseErrorKind::Error,
                        error: DiplomatOption::from(Some(JSONParseError::from(inner))),
                    }
                })
            })?;

            let ff = Self {
                inner,
                _marker: core::marker::PhantomData,
            };

            Ok(Box::new_in(ff, ctx.allocator))
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

            let metadata = &self.inner.borrow_dependent().file_metadata;

            let result_str = match field {
                FoldMetadataQuery::Creator => metadata.creator.as_ref(),
                FoldMetadataQuery::Author => metadata.author.as_ref(),
                _ => unreachable!(),
            }
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

        pub fn frame<'a>(&'a self, index: u16) -> Option<Box<FoldFrame<'a>>> {
            self.inner
                .borrow_dependent()
                .frame(index)
                .map(|inner| Box::new(FoldFrame { inner }))
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
        Inheriting,
    }

    /// A reference to a fold frame
    #[diplomat::opaque]
    #[diplomat::rust_link(fold::FrameRef, Struct)]
    #[derive(Debug, Clone, Copy)]
    pub struct FoldFrame<'fold> {
        inner: fold::FrameRef<'fold>,
    }

    use fold::{Frame, FrameEdges, FrameFaces, FrameVertices};

    pub enum UniformCopyError {
        /// Destination alignment
        DestinationAlignmentInvalid,
    }

    /// A uniform member (like edges_vertices, edges_assignment, edges_length, edges_foldAngle)
    /// As there is a known size per element, there is no indices to be exposed
    #[diplomat::macro_rules]
    macro_rules! define_member_u {
        ($kind:ident, $member:ident, $access:ident, $copy:ident, $scalar:ty) => {
            pub fn $access<'s>(&'s self) -> &'s [$scalar] {
                super::access_u(
                    (&self.inner)
                        .$kind()
                        .$member()
                        .as_ref()
                        .map(|v| v.as_slice())
                        .unwrap_or(&[]),
                    0,
                )
            }

            pub fn $copy(&self, dst: &mut [$scalar], offset: u32) -> Result<u32, UniformCopyError> {
                super::copy_u(
                    dst,
                    (&self.inner)
                        .$kind()
                        .$member()
                        .as_ref()
                        .map(|v| v.as_slice())
                        .unwrap_or(&[]),
                    offset,
                )
            }
        };
    }

    #[diplomat::macro_rules]
    macro_rules! define_member_nu {
        ($kind:ident, $member:ident, $backing:ident, $indices:ident, $copy:ident, $scalar:ty) => {
            /// This copies the raw values behind the coordinates
            /// While they may all be 2D or 3D, this is not by default guaranteed
            /// Some fold files may mix those
            // TODO: When supported by Diplomat, return a special Non-uniform span which also
            // contains the range slices
            //#[diplomat::attr(auto, getter = $backing)]
            pub fn $backing<'s>(&'s self) -> &'s [$scalar] {
                (&self.inner)
                    .$kind()
                    .$member()
                    .as_ref()
                    .map(|v| v.backing.as_slice())
                    .unwrap_or(&[])
            }

            // This requires a backend that supports slices of structs ('abi_compatible')
            #[diplomat::attr(not(supports = abi_compatibles), disable)]
            //#[diplomat::attr(auto, getter = "vertices_coords_indices")]
            pub fn $indices<'s>(&'s self) -> &'s [RawSpan] {
                let original = (&self.inner)
                    .$kind()
                    .$member()
                    .as_ref()
                    .map(|v| v.indices.as_slice())
                    .unwrap_or(&[]);
                bytemuck::cast_slice(original)
            }

            /// If an indices destination is given (len > 0), then this copies up to that destination buffer size of indices, using the provided offset,
            /// as well as the matching backing data, if the destination backing buffer is provided, limited to its capacity.
            ///
            /// If no indices destination is given (len == 0), the offset is applied to the backing data and the it is copied up to the capacity of the destination backing buffer.
            #[diplomat::attr(not(supports = abi_compatibles), disable)]
            pub fn $copy(
                &self,
                backing_dst: &mut [$scalar],
                indices_dst: &mut [RawSpan],
                offset: u32,
            ) -> NUCopyInfo {
                super::copy_nu(
                    &self.$backing(),
                    &self.$indices(),
                    backing_dst,
                    indices_dst,
                    offset,
                )
            }
        };
    }

    impl<'f> FoldFrame<'f> {
        pub fn kind(&self) -> FoldFrameKind {
            match self.inner {
                fold::FrameRef::Key(_) => FoldFrameKind::Key,
                fold::FrameRef::NonInheriting { .. } => FoldFrameKind::NonInheriting,
                fold::FrameRef::Inheriting(_) => FoldFrameKind::Inheriting,
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
            Box::new(VerticesIterator {
                inner: *self,
                cursor: 0,
            })
        }

        pub fn iterate_edges(&self) -> Box<EdgesIterator<'f>> {
            Box::new(EdgesIterator {
                inner: *self,
                cursor: 0,
            })
        }

        pub fn iterate_faces(&self) -> Box<FacesIterator<'f>> {
            Box::new(FacesIterator {
                inner: *self,
                cursor: 0,
            })
        }

        /* access & copy */

        // TODO: Figure out how to use diplomat's macro_rules support to automatize that code for vertices, edges, etc.

        // FIXME: add macros for the uniform ones

        define_member_nu!(
            vertices,
            coords,
            vertices_coords_backing,
            vertices_coords_indices,
            vertices_coords_copy,
            f32
        );
        define_member_nu!(
            vertices,
            edges,
            vertices_edges_backing,
            vertices_edges_indices,
            vertices_edges_copy,
            u32
        );
        // FIXME: handle the option
        //define_member_nu!(vertices, faces, vertices_faces_backing, vertices_faces_indices, vertices_faces_copy, f32);

        // this is uniform, we don't need the non-uniform support
        define_member_u!(edges, vertices, edges_vertices, edges_vertices_copy, u32);
        // FIXME: handle the option
        //define_member_nu!(edges, faces, edges_faces_backing, edges_faces_indices, edges_faces_copy, u32);
        define_member_nu!(
            faces,
            vertices,
            faces_vertices_backing,
            faces_vertices_indices,
            faces_vertices_copy,
            u32
        );
        define_member_nu!(
            faces,
            edges,
            faces_edges_backing,
            faces_edges_indices,
            faces_edges_copy,
            u32
        );
        define_member_nu!(
            faces,
            uvs,
            faces_uvs_backing,
            faces_uvs_indices,
            faces_uvs_copy,
            u32
        );
    }

    #[derive(Debug, Clone, Copy)]
    pub struct NUCopyInfo {
        pub backing_written: u32,
        pub indices_written: u32,
    }

    /// This is a cursor into the flattened values given by the raw_ methods on a frame
    #[diplomat::attr(auto, abi_compatible)]
    #[derive(Debug, Clone, Copy, bytemuck::AnyBitPattern)]
    pub struct RawSpan {
        pub start: u32,
        pub length: u32,
    }

    #[diplomat::opaque]
    pub struct VerticesIterator<'frame> {
        inner: FoldFrame<'frame>,
        cursor: fold::VertexIndex,
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
        pub to: u32,
    }

    #[diplomat::opaque]
    pub struct EdgesIterator<'frame> {
        inner: FoldFrame<'frame>,
        cursor: fold::EdgeIndex,
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
        cursor: fold::FaceIndex,
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
        FaceCount,
    }

    pub enum FoldFrameFloatQuery {
        /// corresponds to vertices_coords
        VerticesCoords,
        /// corresponds to edges_foldAangle
        EdgesFoldAngle,
        /// corresponds to edges_length
        EdgesLength,
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
