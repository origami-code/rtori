# `rtori-core`: core functionality for parsing, interpreting & simulating origami

## Architecture

On non-web platforms, `rtori-core` links to shared libraries.

### Simulation

Simulation is provided by two backends:
- `rtori-os-simd` : packed/portable SIMD implementation of the `OrigamiSimulator` algorithm
- `rtori-core-wgpu` : gpu-based compute implementation (Vulkan, Metal, DX12, WebGPU)

It would be ideal to have a way t
