// Reimplementation of wgpu::util::download_buffer to make it debuggable
/// CPU accessible buffer used to download data back from the GPU.
pub struct DownloadBuffer {
    _gpu_buffer: Arc<super::Buffer>,
    mapped_range: Box<dyn crate::context::BufferMappedRange>,
}

impl DownloadBuffer {
    /// Asynchronously read the contents of a buffer.
    pub fn read_buffer(
        device: &super::Device,
        queue: &super::Queue,
        buffer: &super::BufferSlice<'_>,
        callback: impl FnOnce(Result<Self, super::BufferAsyncError>) + Send + 'static,
    ) {
        let size = match buffer.size {
            Some(size) => size.into(),
            None => buffer.buffer.map_context.lock().total_size - buffer.offset,
        };

        #[allow(clippy::arc_with_non_send_sync)] // False positive on emscripten
        let download = Arc::new(device.create_buffer(&super::BufferDescriptor {
            size,
            usage: super::BufferUsages::COPY_DST | super::BufferUsages::MAP_READ,
            mapped_at_creation: false,
            label: None,
        }));

        let mut encoder =
            device.create_command_encoder(&super::CommandEncoderDescriptor { label: None });
        encoder.copy_buffer_to_buffer(buffer.buffer, buffer.offset, &download, 0, size);
        let command_buffer: super::CommandBuffer = encoder.finish();
        queue.submit(Some(command_buffer));

        download
            .clone()
            .slice(..)
            .map_async(super::MapMode::Read, move |result| {
                if let Err(e) = result {
                    callback(Err(e));
                    return;
                }

                let mapped_range = crate::context::DynContext::buffer_get_mapped_range(
                    &*download.context,
                    download.data.as_ref(),
                    0..size,
                );
                callback(Ok(Self {
                    _gpu_buffer: download,
                    mapped_range,
                }));
            });
    }
}