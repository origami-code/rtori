#![feature(debug_closure_helpers)]
#![feature(generic_arg_infer)]
pub use wgpu;

mod layout;
use layout::PipelineSetLayout;

mod extractor;
mod extractor_gpu;
mod loader;

mod storage_buffers;
mod uniform_buffers;

mod model_size;
pub use model_size::ModelSize;

mod bind_groups;

mod state;
use state::{State, StateLoader};

mod input;
pub use input::Input;

#[derive(Debug)]
pub struct Runner<'device> {
    device: &'device wgpu::Device,
    layout: PipelineSetLayout,
    state: Option<State<'device>>,
}

impl<'device> Runner<'device> {
    pub const WGPU_FEATURES_REQUIRED: wgpu::Features = wgpu::Features::empty();

    pub const fn optimize_features(
        supported: wgpu::Features,
        base: Option<wgpu::Features>,
        adapter_info: &wgpu::AdapterInfo,
    ) -> wgpu::Features {
        let mut base = if let Some(base) = base {
            base
        } else {
            Self::WGPU_FEATURES_REQUIRED
        };

        let result = wgpu::Features::from_bits_truncate(
            base.bits()
                | if supported.contains(wgpu::Features::MAPPABLE_PRIMARY_BUFFERS)
                    && matches!(adapter_info.device_type, wgpu::DeviceType::IntegratedGpu)
                {
                    wgpu::Features::MAPPABLE_PRIMARY_BUFFERS.bits()
                } else {
                    0
                }, /*
                   | if supported.contains(wgpu::Features::PUSH_CONSTANTS) {
                       wgpu::Features::PUSH_CONSTANTS
                   } else { 0 }*/
        );

        result
    }

    pub const WGPU_LIMITS_REQUIRED: wgpu::Limits = {
        let mut limits = wgpu::Limits::downlevel_defaults();
        limits.max_storage_buffers_per_shader_stage = 9;
        limits
    };

    pub const fn optimize_limits(
        supported: wgpu::Limits,
        base: Option<wgpu::Limits>,
    ) -> Result<wgpu::Limits, ()> {
        if supported.max_storage_buffers_per_shader_stage < 9 {
            return Err(());
        }

        let mut result = if let Some(base) = base {
            base
        } else {
            Self::WGPU_LIMITS_REQUIRED
        };

        // We want to have the lowest alignment requirements
        result.min_storage_buffer_offset_alignment = supported.min_storage_buffer_offset_alignment;
        result.min_uniform_buffer_offset_alignment = supported.min_uniform_buffer_offset_alignment;

        Ok(result)
    }

    pub fn create(device: &'device wgpu::Device) -> Self {
        Self {
            device,
            layout: PipelineSetLayout::new(device),
            state: None,
        }
    }

    pub fn prepare(&mut self, size: ModelSize) {
        let state = if let Some(mut state) = self.state.take() {
            state.clear();

            // I'm ignoring it as i'm reloading anyway
            let _should_reload = state.recreate(size, &self.layout);
            state
        } else {
            // Create it
            State::create(self.device, size, &self.layout)
        };

        self.state = Some(state);
    }

    /// maps for loading
    pub fn load(&mut self, size: ModelSize) -> StateLoader<'_> {
        self.prepare(size);

        let state = self
            .state
            .as_mut()
            .expect("We just prepared, thus the state exists (but is not loaded yet)");

        // Load the state
        state.load().unwrap()
    }

    pub fn step(&self, count: u64) -> Option<wgpu::CommandBuffer> {
        let state = if let Some(state) = &self.state {
            state
        } else {
            return None;
        };

        let mut command_encoder =
            self.device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
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

    pub fn extract(
        &mut self,
        queue: &wgpu::Queue,
        kind: crate::state::ExtractFlags,
        callback: impl FnOnce(Result<crate::extractor::ExtractorMappedTarget<'_>, wgpu::BufferAsyncError>)
            + wgpu::WasmNotSend
            + 'static,
    ) -> Result<bool, ()> {
        self.state
            .as_mut()
            .ok_or(())
            .and_then(|state| state.extract(queue, kind, callback))
    }
}
