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

Devalang is a compact domain-specific language (DSL) for music makers, sound designers, and creative coders.
Compose loops, control samples, synthesize audio, and render your ideas — all in clean, readable text.

Whether you're prototyping a beat, building generative music, or performing live, Devalang gives you rhythmic precision with the elegance of code.

From studio sketches to live sets, Devalang puts musical ideas into motion.

> **🚀 v0.1.0 - Complete Rewriting**
>
>
> **NEW**: [Devalang Playground V2.0 is now available](https://playground.devalang.com) — Try it in your browser!

---

## 📚 Quick Access

- [▶️ Playground](https://playground.devalang.com) — Try Devalang in your browser
- [📖 Documentation](https://docs.devalang.com) — Complete language reference
- [🧩 VSCode Extension](https://marketplace.visualstudio.com/items?itemName=devaloop.devalang-vscode) — Syntax highlighting & snippets
- [📜 Changelog](./docs/CHANGELOG.md) — Version history
- [💡 Examples](./examples/)
- [🌐 Website](https://devalang.com) — Project homepage
- [📦 npm Package](https://www.npmjs.com/package/@devaloop/devalang)
- [📦 Rust Crate](https://crates.io/crates/devalang)

---

## ⚡ Quick Start

### Try in Your Browser

> **[Launch the Playground](https://playground.devalang.com)** to try Devalang without installing anything.

### Install via npm (Recommended)

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

---

## 🎵 Your First Devalang Script

Create a file `hello.deva`:

```deva
# Set the tempo
bpm 120

# Load a bank of sounds
bank devaloop.808 as drums

# Create a simple drum pattern
pattern kickPattern with drums.kick = "x--- x--- x--- x---"
pattern snarePattern with drums.snare = "---- x--- ---- x---"
pattern hihatPattern with drums.hihat = "x-x- x-x- x-x- x-x-"

# Play the patterns
call kickPattern
call snarePattern
call hihatPattern
```

### Build the audio

```bash
# Build to WAV
devalang build --path hello.deva --formats wav

# Output: ./output/audio/hello.wav
```

# Play the audio

```bash
# Play the audio file
devalang play --input hello.deva

# Play live (repeats and watch until stopped)
devalang play --live --input hello.deva
```

---

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
- ⚠️ **MP3/FLAC** — Planned for v0.1.1

### 🎯 **Performance**
- ⚡ **Fast builds** — 7-10ms for typical projects
- ⚡ **Low latency** — Optimized audio engine
- ⚡ **Release builds** — 5-6x faster than debug

### 📚 **Learning Resources**
- ✅ **Online Docs** — Complete language reference
- ✅ **VSCode Extension** — Syntax highlighting

---

## 💡 Why Devalang?

- 🎹 **Prototype audio ideas** without opening a DAW
- 💻 **Integrate sound** into code-based workflows
- 🎛️ **Control audio parameters** with readable syntax
- 🧪 **Build musical logic** with variables and conditions
- 🔄 **Create patterns** with expressive notation
- 🎨 **Live code** with fast iteration cycles
- 📦 **Version control** your music with git

---

## 📖 Documentation

**[Visit docs.devalang.com](https://docs.devalang.com)** for:
- Complete syntax reference
- API documentation
- WASM integration guide
- CLI command reference
- Advanced tutorials
- Best practices

---

## 🔧 Development

### Build from Source

```bash
# Clone the repository
git clone https://github.com/devaloop-labs/devalang.git
cd devalang

# Build CLI (Rust)
cargo build --release --features cli

# Build WASM
cargo build --release --features wasm --lib

# Build TypeScript
npm install
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

---

## 📜 License

MIT License — See [LICENSE](./LICENSE) for details.

Copyright (c) 2025 Devaloop

---

<div align="center">
    <strong>Made with ❤️ by the Devaloop team</strong>
    <br />
    <sub>Star ⭐ the repo if you like it!</sub>
</div>
