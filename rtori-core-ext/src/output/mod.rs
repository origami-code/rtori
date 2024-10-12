use std::mem::MaybeUninit;

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct ArrayOutput<T> {
    /// Shall point to a block of memory able to hold [`ArrayOutput::buffer_size`] amount of items.
    /// As an special case, it may be null in case the latter is `0`.
    /// After being written, [`ArrayOutput::output_size`] will contain one of two values depending on whether
    /// this was a null pointer:
    /// - If it was null, [`ArrayOutput::output_size`] will be filled in the required buffer size to complete the requested
    /// copy
    /// - Otherwise, the amount written will be written there
    pub buffer: Option<core::ptr::NonNull<T>>,

    /// The size is in item sizes of T
    /// Shall be less than or equal to the number of items in [`ArrayOutput::buffer`], or `0` if the latter is null
    pub buffer_size: usize,

    /// This must not be null. It will be filled in with either:
    /// - the actual amount written to [`ArrayOutput::buffer`] in case it was non-null
    /// - the required size to be able to execute this copy if [`ArrayOutput::buffer`] was null
    pub written_size: core::ptr::NonNull<usize>,

    pub offset: u32,
}

impl<T: bytemuck::AnyBitPattern> ArrayOutput<T> {
    #[inline]
    unsafe fn copy_from<F>(&mut self, source_len: usize, apply: F)
    where
        F: for<'a> FnOnce(&'a mut [MaybeUninit<T>], usize, usize),
    {
        assert!(
            self.buffer_size == 0 || self.buffer.is_some(),
            "Given buffer be non-null in case the buffer size is more than 0"
        );
        let offset = usize::try_from(self.offset).unwrap();

        match self.buffer {
            // If we have a null buffer, we write out the size it should actually be
            None => {
                unsafe { self.written_size.as_uninit_mut() }.write(source_len - offset);
            }
            // If we have a non-null buffer but the buffer size given is 0, it still is safe (nothing will be copied)
            Some(buffer) => {
                let output_size = usize::min(source_len - offset, self.buffer_size);
                unsafe { self.written_size.as_uninit_mut() }.write(output_size);

                let output_slice =
                    core::ptr::NonNull::slice_from_raw_parts(buffer, self.buffer_size);

                // SAFETY:
                // Safe as long as the given [`ArrayOutput::buffer_size`] is big enough to contain
                // As the output is uninit, does not require that the data in the buffer be initialized
                let output_uninit = unsafe { output_slice.as_uninit_slice_mut() };

                let destination = &mut output_uninit[..output_size];
                assert_eq!(output_size, destination.len());

                apply(destination, offset, output_size);
            }
        }
    }

    /// SAFETY: for this to be safe, the [`ArrayOutput::data`] member must point to an array of memory able to hold at least [`ArrayOutput::output_size`] members of type `T`
    #[inline]
    pub(crate) unsafe fn copy_from_source(&mut self, source: &[T]) {
        let len = source.len();

        let apply = move |destination: &mut [MaybeUninit<T>], offset, len| {
            let source = &source[offset..offset + len];
            assert_eq!(len, source.len());
            core::mem::MaybeUninit::copy_from_slice(destination, source);
        };

        unsafe {
            self.copy_from(len, apply);
        }
    }

    #[inline]
    pub(crate) unsafe fn extend_from_source<I: ExactSizeIterator<Item = T>>(
        &mut self,
        source: I,
        offset_already: bool,
    ) {
        let offset = self.offset as usize;

        let source = source.skip(if offset_already { 0 } else { offset });

        let len = source.len();
        let apply = move |destination: &mut [MaybeUninit<T>], _, len| {
            destination
                .iter_mut()
                .zip(source.take(len))
                .for_each(|(dst, src)| {
                    dst.write(src);
                });
        };

        unsafe {
            self.copy_from(len, apply);
        }
    }

    /// SAFETY: See requirements of [`ArrayOutput::copy_from_source`].
    /// If the source is None, it is treated as being zero-sized
    #[inline]
    pub(crate) unsafe fn copy_from_source_opt(&mut self, source: Option<&[T]>) {
        let source = source.unwrap_or(&[]);
        unsafe { self.copy_from_source(source) }
    }

    #[inline]
    pub(crate) unsafe fn set_empty(&mut self) {
        unsafe { self.written_size.as_uninit_mut() }.write(0);
    }
}

pub type ArrayOutputU8 = ArrayOutput<u8>;
pub type ArrayOutputVec3F = ArrayOutput<[f32; 3]>;
pub type ArrayOutputVec2F = ArrayOutput<[f32; 2]>;
pub type ArrayOutputVec3U = ArrayOutput<[u32; 3]>;
pub type ArrayOutputVec2U = ArrayOutput<[u32; 2]>;
pub type ArrayOutputF32 = ArrayOutput<f32>;

#[repr(C)]
pub union QueryOutput {
    /// Strings are outputted in UTF-8
    pub string_output: ArrayOutputU8,
    pub u16_output: core::ptr::NonNull<u16>,
    pub u32_output: core::ptr::NonNull<u32>,
    pub vec3f_array_output: ArrayOutputVec3F,
    pub vec2f_array_output: ArrayOutputVec2F,
    pub vec3u_array_output: ArrayOutputVec3U,
    pub vec2u_array_output: ArrayOutputVec2U,
    pub vecf_array_output: ArrayOutputF32,
}

impl QueryOutput {
    pub(crate) unsafe fn copy_vecf(&mut self, source: Option<&[f32]>) {
        unsafe { self.vecf_array_output.copy_from_source_opt(source) }
    }

    pub(crate) unsafe fn copy_vec3f(&mut self, source: Option<&[[f32; 3]]>) {
        unsafe { self.vec3f_array_output.copy_from_source_opt(source) }
    }

    pub(crate) unsafe fn extend_vec3f<I: ExactSizeIterator<Item = [f32; 3]>>(&mut self, source: I) {
        unsafe { self.vec3f_array_output.extend_from_source(source, false) }
    }

    pub(crate) unsafe fn empty_vec3f(&mut self) {
        unsafe { self.vec3f_array_output.set_empty() }
    }

    pub(crate) unsafe fn copy_vec2f(&mut self, source: Option<&[[f32; 2]]>) {
        unsafe { self.vec2f_array_output.copy_from_source_opt(source) }
    }

    pub(crate) unsafe fn extend_vec2f<I: ExactSizeIterator<Item = [f32; 2]>>(&mut self, source: I) {
        unsafe { self.vec2f_array_output.extend_from_source(source, false) }
    }

    pub(crate) unsafe fn copy_vec3u(&mut self, source: Option<&[[u32; 3]]>) {
        unsafe { self.vec3u_array_output.copy_from_source_opt(source) }
    }

    pub(crate) unsafe fn extend_vec3u<I: ExactSizeIterator<Item = [u32; 3]>>(&mut self, source: I) {
        unsafe { self.vec3u_array_output.extend_from_source(source, false) }
    }

    pub(crate) unsafe fn empty_vec3u(&mut self) {
        unsafe { self.vec3u_array_output.set_empty() }
    }

    pub(crate) unsafe fn copy_vec2u(&mut self, source: Option<&[[u32; 2]]>) {
        unsafe { self.vec2u_array_output.copy_from_source_opt(source) }
    }

    pub(crate) unsafe fn extend_vec2u<I: ExactSizeIterator<Item = [u32; 2]>>(&mut self, source: I) {
        unsafe { self.vec2u_array_output.extend_from_source(source, false) }
    }

    pub(crate) unsafe fn empty_vec2u(&mut self) {
        unsafe { self.vec2u_array_output.set_empty() }
    }

    pub(crate) unsafe fn copy_str(&mut self, source: Option<&str>) {
        unsafe {
            self.string_output
                .copy_from_source_opt(source.map(|str| str.as_bytes()))
        }
    }
}
