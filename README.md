# `rtori`: (R)eal-(t)ime (Ori)gami

`rtori` is a **toolkit** for interpreting, simulating and visualizing 3D origami.

## Components

### Main

- `rtori-core-ext`: a library that provides a packaged set of functionality for interpreting and simulating 3D origami (shared library: `.dll`, `.so`, `.dylib`)
- `rtori-touchdesigner`: a set of plugins for [touchdesigner](https://derivative.ca) exposing the functionality through several components
- `rtori-cli`: a CLI interface exposing the same functionality

### Supporting

- `rtori-core`: the core behind `rtori-core-ext`, presenting a `rust-based` dylib
- `fold`: a library to parse and execute operations on [`.fold` files](https://edemaine.github.io/fold/doc/spec.html)

## Target Architectures

Potential support: (In LLVM/Rust triplet)

- `thumbv8m.main-none-eavi`: for Raspberry Pi RP2350 (rtori-core only)
- `thumbv6m-none-eabi`: for Raspberry Pi 2040 (rtori-core only)
- `x86_64-pc-windows-msvc`: for Windows on x86-64
- `i686-pc-windows-msvc`: for Windows on x86-32
- `aarch64-pc-windows-msvc`: for Windows on ARM
- `x86_64-apple-darwin`: for MacOs on Intel
- `aarch64-apple-darwin`: for MacOS on Apple Silicon
- `x86_64-unknown-linux-gnu`: for Linux on x86-64 (gnu)
- `x86_64-unknown-linux-musl`: for Linux on x86-64 (musl)
- `arm-unknown-linux-gnueabihf`: for Linux on armv6 (rpi 1)
- `armv7-unknown-linux-gnueabihf`: for Linux on rpi 2 rev 1.0
- `aarch64-unknown-linux-gnu`: for Linux on aarch64 (rpi 2 and later)
- `aarch64-unknown-linux-musl`: for Linux on aarch64 (rpi 2 and later) (musl)
- `riscv32imac-unknown-none-elf`: for rp2350 (riscv cores)
- `wasm32-unknown-unknown`: for web/wasm
