<div align="center">
    <img src="https://devalang.com/images/devalang-logo-min.png" alt="Devalang Logo" width="100" />
</div>

# Changelog

All notable changes to Devalang will be documented in this file.

## Version 0.1.3 - 2025-10-06

### 🛠️ Improvements

- Improved addon management in the CLI.
  - Use `devalang addon discover --local --install` to discover and install local **gzipped** addons (banks, plugins, etc.).
- Modularized audio interpreter to improve maintainability and extensibility.
- Re-implemented plugin usage in scripts with `@use <author>.<name> as <optional_alias>` syntax.
  - Usage with synth: `let mySynth = synth myPlugin.synthVar = { waveform: "saw", ... }`
- Re-implemented `@load` statement usable with both audio files (**only .wav for now**) and MIDI (**.mid**) files.
- Re-implemented `@import` and `@export` statements for modularizing scripts.

### 🐛 Bug Fixes

- Fixed triggers not playing when classic call syntax was used

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
