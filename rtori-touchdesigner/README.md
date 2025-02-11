# `rtori` touchdesigner plugins & components

Supported Touchdesigner version `099` builds `2023` or later.

## Components (`.tox`)

These are the `.tox` most people should use instead of the custom operators directly.
They use some of the plugins while adding functionality using native TouchDesigner OPs.

|Name (prefixed `RTOri`) | Description                                                      |State|
|------------------------|------------------------------------------------------------------|:---:|
| `SimulateCOMP`         | Simulates an origami folding, taking care of translating the UVs |🚧|

## Plugins

|Kind |Name (prefixed `RTOri`) |Description                                              |State|
|-----|------------------------|---------------------------------------------------------|:---:|
|SOP  | `Simulate`             | Simulates an origami folding                            |🚧|
|CHOP | `Simulate`             | Simulates an origami folding                            |🎯|
|TOP  | `Simulate`             | Simulates an origami folding (ouputs positions/velocities/errors) |🎯|
|POP *| `Simulate`             | Simulates an origami folding                            |🎯|
|DAT  | `FoldSelectFrame`      | Parses a fold file, outputs a single frame              |🎯|
|DAT  | `FoldSelect`           | Parses a fold file/frame, outputs the requested fields  |🎯|
|DAT  | `FoldAdd`              | Adds vertices to a fold file, possibly linking them     |🎯|
|DAT  | `SOPToFold`            | Converts a surface (SOP) into a fold file               |🎯|
|POP *| `POPToFold`            | Converts a surface (POP) into a fold file               |🎯|

(*):`POP` or Particle Operator(s) haven't been released yet. They promise increased performance for surface operations.


All the `Simulate` custom operators can be either linked to a fold file,
or to another one which is linked to a fold file, so as to cheapily add a new output from the same simulation.

All the `..Fold..` custom operators also provide access to python queries, read-only.

## Platform/Architecture Support

|OS       |Architecture|State|Note |
|:-------:|:----------:|:---:|:----|
|Windows  |x86_64      |✅   ||
|Windows  |arm64ec(1)  |🎯   ||
|macOS    |X86_64      |🎯   |10.15+(2)|
|macOS    |arm64       |🎯   |10.15+(2)|

🎯: a target, not currently running
(1): 
(2): minimum version supported by TouchDesigner 2023

## Building

### `macOS`

#### Requirements

- XCode 15.2 or later
- cmake 3.30 or later
- rustup 
- git

Install via brew: `brew install cmake rustup`

#### Setup

Run `git submodule update --init --recursive` after cloning this repository.

```sh
# Setup a nightly toolchain
rustup default nightly

# Add the target(s) you want, EITHER
rustup target add x86_64-apple-darwin # x86_64 
rustup target add aarch64-apple-darwin # arm64/apple silicon
```

And [create a code signing certificate](https://www.simplified.guide/macos/keychain-cert-code-signing-create) if you don't already have one.

#### Build

Replace `arm64` by `x86_64` if building for an intel mac.

```sh
cmake --preset macOS-arm64 -DMACOS_CODE_SIGNING_IDENTITY="YOUR_IDENTITY" # Configure
cmake --build --preset macOS-arm64 --config RelWithDebInfo # Build
cmake --install macOS-arm64-build --config RelWithDebInfo # Install to `macOS-arm64-output`
# The folder `macOS-arm64-output` now contains the plugins, and can be copied and renamed to `Plugins/` inside the target projects
```

## Implementation Notes

### Architecture

The plugins depend on a common library `rtori-td-common.{dll,dylib}`, providing core functionality in a TD-compatible outfit, with dynamic/shared linkage. This also reduces the size of the plugins.

#### `RTOriSimulate...`

All the `RTOriSimulate...` plugins depend on another shared library,
`RTOriSimulateShared`, which provides the base class.

This allows the different plugins to cast the objects of the other classes (`RTOriSimulate...`)
to a common one which provides the required operations to access the simulation.

### Windows on Arm64

For Windows on arm64, there is currently (2025-01-21) no native release for arm64.
Some people have had success with the built-in emulation from Windows.

To accelerate the CPU-heavy code of this plugin, we could build it as well as dependencies as [`ARM64EC`](https://learn.microsoft.com/en-us/windows/arm/arm64ec).

### macOS

There should be no troubles besides testing.

## References

<https://github.com/TouchDesigner/CustomOperatorSamples/tree/main/SOP/GeneratorSOP>
<https://docs.derivative.ca/CPlusPlus_SOP>
<https://docs.derivative.ca/Write_a_CPlusPlus_Plugin>
