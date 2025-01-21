# `rtori` touchdesigner plugins

## Plugins

|Kind |Name              |Description                     |State|
|:---:|:----------------:|:------------------------------:|:---:|
|SOP  | RTOriSimulateSOP | Simulates an origami folding  :|🚧|
|CHOP | RTOriSimulateCHOP| Simulates an origami folding  :|🎯|
|TOP  | RTOriSimulateTOP | Simulates an origami folding  :|🎯|
|POP *| RTOriSimulatePOP | Simulates an origami folding  :|🎯|

All the `RTOriSimulate...` plugins can be either linked to a fold file,
or to another one which is linked to a fold file, so as to cheapily add a new output from the same simulation.

(*):`POP` or Particle Operator(s) haven't been released yet. They promise increased performance for surface operations.

## Platform/Architecture Support

|OS       |Architecture|State|
|:-------:|:----------:|:---:|
|Windows  |x86_64      |✅   |
|Windows  |arm64       |🎯   |
|macOS    |X86_64      |🎯   |
|macOS    |arm64       |🎯   |

🎯: a target, not currently running

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
