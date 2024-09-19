pub use wgpu;

mod layout;
use layout::PipelineSetLayout;

mod storage_buffers;
mod uniform_buffers;

mod model_size;
pub use model_size::ModelSize;

mod bind_groups;

mod state;
use state::State;

mod input;
pub use input::Input;

#[derive(Debug)]
pub struct Runner {
    layout: PipelineSetLayout,
    state: Option<State>,
}

impl Runner {
    pub const WGPU_FEATURES_REQUIRED: wgpu::Features = wgpu::Features::empty();
    pub const WGPU_FEATURES_OPTIMIZED: wgpu::Features =
        Self::WGPU_FEATURES_REQUIRED;
        //| wgpu::Features::MAPPABLE_PRIMARY_BUFFERS
        //| wgpu::Features::PUSH_CONSTANTS
        //| wgpu::Features::SHADER_I16;

    pub const WGPU_LIMITS_REQUIRED: wgpu::Limits = {
        let mut limits = wgpu::Limits::downlevel_defaults();
        limits.max_storage_buffers_per_shader_stage = 9;
        limits
    };

    pub const fn optimize_limits(supported: wgpu::Limits, base: Option<wgpu::Limits>) -> Result<wgpu::Limits, ()> {
        if supported.max_storage_buffers_per_shader_stage < 9 {
            return Err(());
        } 

        
        let mut result = if let Some(base) = base { base} else {Self::WGPU_LIMITS_REQUIRED};
        result.min_storage_buffer_offset_alignment = supported.min_storage_buffer_offset_alignment;
        result.min_uniform_buffer_offset_alignment = supported.min_uniform_buffer_offset_alignment;
        Ok(result)
    }

    pub fn create(device: &wgpu::Device) -> Self {
        Self {
            layout: PipelineSetLayout::new(device),
            state: None,
        }
    }

    pub fn prepare(&mut self, device: &wgpu::Device, params: &ModelSize) {
        // Already prepared, just reset
        let reused = {
            let existing = self.state.take();
            if let Some(mut existing) = existing {
                if existing.params() == params {
                    existing.clear();
                    Some(existing)
                } else {
                    drop(existing);
                    None
                }
            } else {
                None
            }
        };

        let new_state = reused.unwrap_or_else(|| State::create(device, *params, &self.layout));

        self.state = Some(new_state);
    }

    pub fn load<I: Input>(&mut self, data: I) {
        todo!()
    }

    pub fn step(&self, device: &wgpu::Device, count: u64) -> Option<wgpu::CommandBuffer> {
        let state = if let Some(state) = &self.state {
            state
        } else {
            return None;
        };

        let mut command_encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("rtori-command_encoder_steppings"),
        });
        let mut pass = command_encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: Some("rtori-compute_pass-steppings"),
            timestamp_writes: None,
        });
        for _i in 0..count {
            state.encode_pass(&self.layout, &mut pass);
        }
        Some(command_encoder.finish())
    }
}
