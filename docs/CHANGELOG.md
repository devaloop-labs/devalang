<div align="center">
    <img src="https://devalang.com/images/devalang-logo-min.png" alt="Devalang Logo" width="100" />
</div>

# Changelog

All notable changes to Devalang will be documented in this file.

## Version 0.1.5 - 2025-10-25

### 🚀 What's New

#### Routing

#### Automation (global & notes) [(See automation examples for more details)](../examples/automation)

Get more expressive with parameter automation using curves !

- Implemented `automate <instrument_name> mode <global|note>:` statement to automate parameters using curves.
  - **Global mode**: automates all notes played by the instrument.
  - **Notes mode**: automates each of the notes defined inside the block.
- Implemented built-in curves:
  - `$curve.linear` : default linear curve.
  - `$curve.easeIn` : quadratic ease-in curve.
  - `$curve.easeOut` : quadratic ease-out curve.
  - `$curve.easeInOut` : quadratic ease-in-out curve.
- Implemented advanced curves:
  - `$curve.swing(<intensity>)` : sinusoidal swing curve with optional intensity (0.0 to 1.0).
  - `$curve.bounce(<height>)` : bouncing curve with optional height (0.0 to 1.0).
  - `$curve.elastic(<intensity>)` : elastic curve with optional intensity (0.0 to 2.0+).
  - `$curve.bezier(<x1>, <y1>, <x2>, <y2>)` : cubic Bezier curve with 4 control points.
  - `$curve.step(<steps>)` : stepped curve with specified number of steps.
  - `$curve.random` : random curve generating random values between 0.0 and 1.0.
  - `$curve.perlin` : Perlin noise curve for organic variations.
- Implemented LFO support in synths for modulation effects.
  - Added `lfo` parameter in synth definitions with `rate`, `depth`, `shape`, and `target` options.

### 🐛 Bug Fixes

- Fixed MIDI rendering issue that was rendering empty MIDI files.

## Version 0.1.4 - 2025-10-14

### 🚀 What's New

#### MIDI devices support I/O (beta) [(See mapping examples for more details)](../examples/routing/mapping.deva)

Generate notes and patterns using your favorite MIDI controller !

- Implemented MIDI devices `mapping` statement inside scripts for real-time control.
  - Added `mapping` global object to access MIDI device information and states.
  - Added `on` event handler to react to MIDI events in scripts. (e.g. `on mapping.out.<device_name>.noteOn:`).
  - Possibility to bind `pattern` and `synth` from MIDI events. (e.g. `bind <pattern_name> -> mapping.out.<device_name> with { port: 1, channel: 10 }`).
- Implemented MIDI devices `list` command in the CLI to list available MIDI devices.
- Implemented MIDI devices `preview` command in the CLI to preview MIDI data from a device.
  - Added `--port <port_number>` to preview MIDI data from a specific port.
- Implemented MIDI devices `write` command in the CLI to write MIDI data inside scripts.
  - Added `--out <file_path>` to specify the file to write in.
  - Added `--mode <mode>` to specify the write mode ('synth' or 'pattern').
  - Added `--port <port_number>` to specify the input port.
  - Added `--step <step>` to specify the step duration as fraction. (e.g. `1/16` for sixteenth note).
  - Added `--bpm <bpm>` to specify the BPM for the MIDI file (default: 120).
  - Added `--live` flag to write MIDI data in real-time (default: false).
  - Added `--rewrite` flag to overwrite existing file (default: false).
  - Added `--velocity-zero-as-rest` flag to write notes with velocity 0 as `rest/sleep` (default: true).
  - **Synth mode**: writes MIDI note events as `synth` calls.
    - Added `--synth-name <synth_name>` to specify the synth name.
    - Added `--waveform <waveform>` to specify the waveform for the generated synth (default: "saw").
    - Added `--group <group_name>` to specify the group name for the generated synth.
  - **Pattern mode**: writes MIDI note events as `pattern` definitions.
    - Added `--pattern-name <pattern_name>` to specify the pattern name.
    - Added `--trigger <trigger_name>` to specify the trigger name for the generated pattern.

### 🛠️ Improvements

- Added support for notes like "Bb4" [(see issue #3)](https://github.com/devaloop-labs/devalang/issues/3).
- Removed temporary debug `println!` statements and replaced useful logs with structured `Logger` calls.
- Fixed unused imports and variables warnings in MIDI native module.

### 🐛 Bug Fixes

- Fixed issue with inline `pattern` not working as expected.
- Fixed `synth` and `pattern` not working when called together with `spawn`.
- Patched CLI dependencies to include mp3lame and exclude it from the WASM build.

## Version 0.1.3 - 2025-10-08

### 🛠️ Improvements

- Improved addon management in the CLI.
  - Use `devalang addon discover --local --install` to discover and install local **gzipped** addons (banks, plugins, etc.).
- Modularized audio interpreter to improve maintainability and extensibility.
- Re-implemented plugin usage in scripts with `@use <author>.<name> as <optional_alias>` syntax.
  - Usage with synth: `let mySynth = synth myPlugin.synthVar = { waveform: "saw", ... }`
- Re-implemented `@load` statement usable with both audio files (**.wav, .mp3, .ogg, .flac**) and MIDI (**.mid**) files.
- Re-implemented `@import` and `@export` statements for modularizing scripts.
- Implemented audio export to **MP3** format.

### 🐛 Bug Fixes

- Fixed triggers not playing when classic call syntax was used
- Fixed unwrap without checking error when parsing wav files
- Fixed unwrap without checking error when parsing print statements

## Version 0.1.2 - 2025-10-05

### 🛠️ Improvements

- Added installers for Windows, macOS and Linux in CI
- Implemented bank samples lazy loading
- Added `pattern` support (inline, direct and with options)
- Added on-the-fly assignation for `synth`

### 🐛 Bug Fixes

- Fixed some not working examples

## Version 0.1.1 - 2025-10-04

Complete rewrite of the codebase with improved architecture, performance, and developer experience.

### 🚀 What's New

- **Audio engine overhaul** with better timing and scheduling
- **Audio rendering** improvements for lower latency and higher fidelity
- **Complete codebase refactoring** for better maintainability and performance
- **Improved architecture** with cleaner separation of concerns
- **Parallel spawn execution** using Rayon for better performance
- **Enhanced type system** with better error handling
- **Modern CLI** with intuitive commands

### 🐛 Bug Fixes

- Fixed issues with audio playback synchronization
- Resolved memory leaks in the WASM module
- Improved error handling for invalid input

### 🛠️ Breaking Changes

- No more config inside note/chord functions; use chained parameters instead
- Config file format now supports JSON and TOML
- Some internal APIs have changed; refer to the updated documentation for details
- CLI commands have been updated; refer to the new documentation for details
- Removed deprecated features and modules for a cleaner codebase
- Removed deprecated devalang_* crates; use the main crate with feature flags instead

---

<div align="center">
    <strong>Made with ❤️ by the Devaloop team</strong>
</div>
