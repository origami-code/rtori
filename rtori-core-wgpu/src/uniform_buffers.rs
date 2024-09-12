use std::num::NonZeroU64;

pub struct UniformBindings<'backing> {
    pub crease_percentage: wgpu::BufferBinding<'backing>,
    pub dt: wgpu::BufferBinding<'backing>,
}

impl UniformBindings<'_> {
    pub const COUNT: usize = 2;
}

pub struct BackingUniformBuffer {
    buffer: wgpu::Buffer,
    uniform_alignment: u64,
}

impl BackingUniformBuffer {
    const fn align_address(cursor: u64, alignment: u64) -> u64 {
        let multiple = u64::div_ceil(cursor, alignment);

        multiple * alignment
    }

    pub const fn min_size(uniform_alignment: u64) -> u64 {
        let crease_percentage_offset = Self::align_address(0, uniform_alignment);
        let crease_percentage_size = 4;

        let dt_offset = Self::align_address(
            crease_percentage_offset + crease_percentage_size,
            uniform_alignment,
        );
        let dt_size = 4;

        let total = dt_offset + dt_size;

        total
    }

    const fn range_crease_percentage(&self) -> (u64, u64) {
        let offset = Self::align_address(0, self.uniform_alignment);
        let range = 4;

        (offset, range)
    }

    const fn range_dt(&self) -> (u64, u64) {
        let (cp_offset, cp_size) = self.range_crease_percentage();

        let offset = Self::align_address(cp_offset + cp_size, self.uniform_alignment);
        let range = 4;

        (offset, range)
    }

    pub fn allocate(device: &wgpu::Device, uniform_alignment: u64) -> Self {
        let min_size = Self::min_size(uniform_alignment);
        let buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("rtori-backing-uniform-buffer"),
            size: min_size,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::UNIFORM,
            mapped_at_creation: true,
        });

        Self {
            buffer,
            uniform_alignment,
        }
    }

    pub fn bindings<'backing>(&'backing self) -> UniformBindings<'backing> {
        let crease_percentage_offset = Self::align_address(0, self.uniform_alignment);
        let crease_percentage_size = 4;

        let crease_percentage_binding = wgpu::BufferBinding {
            buffer: &self.buffer,
            offset: crease_percentage_offset,
            size: Some(NonZeroU64::try_from(crease_percentage_size).unwrap()),
        };

        let dt_offset = Self::align_address(
            crease_percentage_offset + crease_percentage_size,
            self.uniform_alignment,
        );
        let dt_size = 4;

        let dt_binding = wgpu::BufferBinding {
            buffer: &self.buffer,
            offset: dt_offset,
            size: Some(NonZeroU64::try_from(dt_size).unwrap()),
        };

        UniformBindings {
            crease_percentage: crease_percentage_binding,
            dt: dt_binding,
        }
    }
}
