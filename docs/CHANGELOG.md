<div align="center">
    <img src="https://devalang.com/images/devalang-logo-min.png" alt="Devalang Logo" width="100" />
</div>

# Changelog

All notable changes to Devalang will be documented in this file.

## Version 0.1.0 - 2025-10-04

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
