<div align="center">
    <img src="https://devalang.com/images/devalang-logo-min.png" alt="Devalang Logo" width="100" />
</div>

![Rust](https://img.shields.io/badge/Made%20with-Rust-orange?logo=rust)
![TypeScript](https://img.shields.io/badge/Built%20with-TypeScript-blue?logo=typescript)
![Node.js](https://img.shields.io/badge/Node.js-16%2B-brightgreen?logo=node.js)

![Project Status](https://img.shields.io/badge/status-preview-blue)
![Version](https://img.shields.io/npm/v/@devaloop/devalang)
![License: MIT](https://img.shields.io/badge/license-MIT-green)

![Linux](https://img.shields.io/badge/linux-supported-blue?logo=linux)
![macOS](https://img.shields.io/badge/macOS-supported-blue?logo=apple)
![Windows](https://img.shields.io/badge/windows-supported-blue?logo=windows)

![npm](https://img.shields.io/npm/dt/@devaloop/devalang)
![crates](https://img.shields.io/crates/d/devalang)

# 🦊 Devalang — Write music with code

Devalang is a compact **domain-specific language** (DSL) for **music makers**, **sound designers**, and **creative coders**.
Compose loops, control samples, synthesize audio, and render your ideas — all in clean, **readable text**.

Whether you're prototyping a beat, building **generative music**, or **performing live**, Devalang gives you rhythmic precision with the elegance of code.

**From studio sketches to live sets, Devalang puts musical ideas into motion.**


> **🚀 v0.1.0+ - Complete Rewriting**
>
> **NEW**: [Devalang Playground V2.0 is now available](https://playground.devalang.com) — Try it in your browser!


## 📚 Quick Access

### Websites & Resources
- [🌐 Website](https://devalang.com) — Project homepage
- [▶️ Playground](https://playground.devalang.com) — Try Devalang in your browser
- [📖 Documentation](https://docs.devalang.com) — Complete language reference

### Important files
- [📜 Changelog](./docs/CHANGELOG.md) — Version history
- [💡 Examples](./examples/)

### Common projects and tools
- [📦 Devapack](https://github.com/devaloop-labs/devapack) — Community-driven addons
- [🧩 VSCode Extension](https://marketplace.visualstudio.com/items?itemName=devaloop.devalang-vscode) — Syntax highlighting & snippets

### Downloads
- [🐙 Installers](https://devalang.com/download) — For Windows, macOS, and Linux
- [📦 npm](https://www.npmjs.com/package/@devaloop/devalang) — Install via npm
- [📦 cargo](https://crates.io/crates/devalang) — Install via Cargo

## ⚡ Quick Start

### Try in Your Browser

> **[Launch the Playground](https://playground.devalang.com) to try Devalang without installing anything.**

### Download the Installers (Recommended)

Visit the [Download page](https://devalang.com/download) to get the latest releases for Windows, macOS, and Linux.

### Install via npm (Node.js)

```bash
npm install -g @devaloop/devalang
```

### Install via Cargo (Rust)

```bash
cargo install devalang
```

### Create Your First Project

```bash
# Initialize a new project
devalang init my-project

# Navigate to the project
cd my-project

# Check syntax
devalang check --entry examples/index.deva

# Build audio files
devalang build --path examples/index.deva --formats wav mid

# Play audio (live mode)
devalang play --live --input examples/index.deva
```

## 📦 (optional) Install addons

Devalang supports addons to extend functionalities. This allows you to easily add sound banks, effects, or other features.

> To create your own addon, please refer to the [Devapack documentation](https://github.com/devaloop-labs/devapack/tree/main/docs).

```bash
# List available addons
devalang addon list

# Install an addon (format: <author>.<addon-name>)
devalang addon install devaloop.808
```

This will install the `devaloop.808` sound bank in your current working directory inside `.deva` folder.

**You can then use it in your Devalang scripts !**

## 🎵 Your First Devalang File

Create a file `hello.deva` or `index.deva` (if you do not specify `--input` argument, it defaults to `index.deva`).

#### Nomenclature for .deva files

- Devalang files use the `.deva` extension.
- Devalang engine is **indentation-sensitive** for blocks, similar to Python.
- Files are plain text and can be edited with **any text editor** (VSCode recommended).
- Ensure your text editor supports **UTF-8 encoding**.
- Devalang is **case-sensitive**, so be consistent with capitalization.
- Devalang reads files from **top to bottom**, so order matters.
- Devalang files typically start with global settings (e.g., `bpm`, `bank`), followed by definitions (`synth`, `pattern`, `group`), and finally execution commands (`spawn`, `play`).
- Devalang files can include comments using `#` or `//` for single-line comments.
- You can name your files anything, but `index.deva` is a common convention for the main entry file.
- You can organize your project with subfolders as needed. (use module system like `@import { var } from '<module_path>'` and `@export { var }`).

Refer to the [documentation](https://docs.devalang.com) for a complete syntax reference.

```deva
# Set the tempo
bpm 120

# Load a bank of sounds (make sure you have the bank installed)
bank devaloop.808 as drums

# Create a simple kick pattern
pattern kickPattern with drums.kick = "x--- x--- x--- x---"

# Define a synth and a melody
let mySynth = synth saw

# Define a melody using a group to organize notes
group myMelody:

  mySynth -> note(C5)
      -> duration(500)           # 500ms

  mySynth -> note(E5)
      -> duration(1/4)           # Quarter note

  mySynth -> note(G5)
      -> duration(1/16)           # Sixteenth note
      -> velocity(0.8)            # Velocity (0.0 to 1.0) or 0-127
      -> lpf(800)                 # Lowpass filter at 800Hz
      -> reverb({ size: 0.3 })    # Reverb effect

# Play the melody (in parallel)
spawn myMelody

# Play the kick pattern (in parallel too)
spawn kickPattern
```

### (optional) Configure project settings

You can create a `devalang.json` (recommended) or `devalang.toml` or even `.devalang` (legacy) file to customize check/build/play settings.

This typically evitate to re-type common arguments like `--path`, `--formats`, etc.

> Comments are not supported in config files, please use `devalang init` to generate a default config.

```jsonc
{
  "project": {
    "name": "My Awesome Project"        // Change this to adjust project name
  },
  "paths": {
    "entry": "audio/helloWorld.deva",   // Change this to adjust entry file path
    "output": "output"                  // Change this to adjust output directory
  },
  "audio": {
    "format": ["wav", "mid"],           // Change this to adjust output formats (options: wav, mid, mp3)
    "bit_depth": 16,                    // Change this to 24 or 32 for higher quality
    "channels": 2,                      // Change this to 1 for mono output
    "sample_rate": 44100,               // Change this to 48000 for higher quality
    "resample_quality": "sinc24",       // Change this to adjust resampling quality (options: sinc8, sinc16, sinc24, sinc32)
    "bpm": 120                           // Change this to adjust the project tempo (only if not set in code)
  },
  "live": {
    "crossfade_ms": 500                  // Change this to adjust crossfade duration when playing live
  }
}

```

### Build the audio

```bash
# Build to WAV, MP3, and MIDI
devalang build --path hello.deva --formats wav,mp3,mid
```

### Play the audio

```bash
# Play the audio file
devalang play --input hello.deva

# Play live (repeats and watch until stopped)
devalang play --live --input hello.deva

# Play live loop with very short crossfade
# With 50ms, transitions between loops are no more distinguishable
devalang play --live --crossfade-ms 50 --input hello.deva
```

## 🚀 Features

### 🎵 **Core Language**
- ✅ **Lexer & Parser** — Complete tokenization and AST generation
- ✅ **Patterns** — Rhythmic notation with swing, humanize, velocity
- ✅ **Synths** — Built-in synthesizers with ADSR envelopes
- ✅ **Filters** — Lowpass, highpass, bandpass audio filtering
- ✅ **Effects** — Reverb, delay, distortion, drive, chorus
- ✅ **Variables** — `let`, `const`, `var` with scoping
- ✅ **Groups & Spawn** — Organize and parallelize execution
- ✅ **Loops & Conditions** — `for`, `if`, `else` control flow
- ✅ **Triggers** — Conditional audio triggering
- ✅ **Events** — Event system with `on` and `emit`

### 🛠️ **CLI Tools**
- ✅ `devalang init` — Scaffold new projects (3 templates)
- ✅ `devalang build` — Compile to WAV/MIDI
- ✅ `devalang check` — Validate syntax
- ✅ `devalang play` — Audio playback
- ✅ `devalang addon` — Manage addons (install, list, discover)
- ✅ `devalang login/logout` — Authentication
- ✅ `devalang telemetry` — Privacy controls

### 🌐 **WASM API**
- ✅ `render_audio()` — Browser audio rendering
- ✅ `render_midi_array()` — MIDI export
- ✅ `debug_render()` — Debug information
- ✅ `parse()` — Parse Devalang code
- ✅ TypeScript types included

### 📦 **Output Formats**
- ✅ **WAV** — 16/24/32-bit audio export
- ✅ **MIDI** — Standard MIDI file export
- ✅ **MP3** — Lossy audio export (via LAME)

### 🎯 **Performance**
- ⚡ **Fast builds** — 7-10ms for typical projects
- ⚡ **Low latency** — Optimized audio engine
- ⚡ **Release builds** — 5-6x faster than debug

### 📚 **Learning Resources**
- ✅ **Online Docs** — Complete language reference
- ✅ **VSCode Extension** — Syntax highlighting

## 💡 Why Devalang?

- 🎹 **Prototype audio ideas** without opening a DAW
- 💻 **Integrate sound** into code-based workflows
- 🎛️ **Control audio parameters** with readable syntax
- 🧪 **Build musical logic** with variables and conditions
- 🔄 **Create patterns** with expressive notation
- 🎨 **Live code** with fast iteration cycles
- 📦 **Version control** your music with git

## 📖 Documentation

Visit **[docs.devalang.com](https://docs.devalang.com)** for:
- Complete syntax reference
- API documentation
- WASM integration guide
- CLI command reference
- Advanced tutorials
- Best practices

## 🔧 Development

### Build from Source

```bash
# Clone the repository
git clone https://github.com/devaloop-labs/devalang.git
cd devalang

# NPM (TypeScript) and Cargo (Rust) are required
npm install

# Build CLI (Rust)
cargo build

# Build WASM (Web & Node.js)
npm run rust:wasm:all

# Build TypeScript
npm run ts:build

# Run tests
cargo test --features cli
npm test
```

## 🤝 Contributing

We welcome contributions! See [CONTRIBUTING.md](./CONTRIBUTING.md) for guidelines.

### Ways to Contribute

- 🐛 **Report bugs** via [GitHub Issues](https://github.com/devaloop-labs/devalang/issues)
- 💡 **Suggest features** in discussions
- 📝 **Improve docs** with pull requests
- 🎵 **Share examples** of your creations
- 🧪 **Write tests** for new features

## 📜 License

MIT License — See [LICENSE](./LICENSE) for details.

Copyright (c) 2025 Devaloop

---

<div align="center">
    <strong>Made with ❤️ by <a href="https://labscend.studio">Labscend Studios</a></strong>
    <br />
    <sub>Star ⭐ the repo if you like it !</sub>
</div>
