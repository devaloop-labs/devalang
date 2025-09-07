<div align="center">
    <img src="https://devalang.com/images/devalang-logo-min.png" alt="Devalang Logo" width="100" />
</div>

![Rust](https://img.shields.io/badge/Made%20with-Rust-orange?logo=rust)
![TypeScript](https://img.shields.io/badge/Built%20with-TypeScript-blue?logo=typescript)
![Node.js](https://img.shields.io/badge/Node.js-18%2B-brightgreen?logo=node.js)

![Project Status](https://img.shields.io/badge/status-beta-blue)
![Version](https://img.shields.io/npm/v/@devaloop/devalang)
![License: MIT](https://img.shields.io/badge/license-MIT-green)

![Linux](https://img.shields.io/badge/linux-supported-blue?logo=linux)
![macOS](https://img.shields.io/badge/macOS-supported-blue?logo=apple)
![Windows](https://img.shields.io/badge/windows-supported-blue?logo=windows)

![npm](https://img.shields.io/npm/dt/@devaloop/devalang)
![crates](https://img.shields.io/crates/d/devalang)

![GitHub Actions Workflow Status](https://img.shields.io/github/actions/workflow/status/devaloop-labs/devalang/.github/workflows/ci.yml)

# 🦊 Devalang (CORE) — Compose music with code

Devalang is a tiny domain-specific language (DSL) for music makers, sound designers, and audio hackers.
Compose loops, control samples, render and play audio — all in clean, readable text.

Whether you're building a track, shaping textures, or performing live, Devalang helps you think in rhythms. It’s designed to be simple, expressive, and fast — because your ideas shouldn’t wait.

From studio sketches to live sets, Devalang gives you rhythmic control — with the elegance of code.

> **🚧 Notice 🚧**
>
> Includes synthesis, playback, and rendering features, but is still in early development, and breaking changes may occur.
>
> **NEW**: [Devaforge is now available for creating addons](https://github.com/devaloop-labs/devaforge).
>
> **NEW**: Now available for Windows, Linux, and macOS.

## 📚 Quick Access

- [▶️ Playground](https://playground.devalang.com)
- [📖 Documentation](https://docs.devalang.com)
- [🧩 VSCode Extension](https://marketplace.visualstudio.com/items?itemName=devaloop.devalang-vscode)
- [🎨 Prettier Plugin](https://www.npmjs.com/package/@devaloop/prettier-plugin-devalang)
- [📜 Changelog](./docs/CHANGELOG.md)
- [💡 Examples](./examples/)
- [🌐 Project Website](https://devalang.com)
- [📦 Devaforge on npm](https://www.npmjs.com/package/@devaloop/devaforge)
- [📦 Devalang on npm](https://www.npmjs.com/package/@devaloop/devalang)

## ⏱️ Try it now !

### Try Devalang in your browser

> [Have a look at the Playground to try Devalang directly in your browser](https://playground.devalang.com)

### Try Devalang in your terminal

#### With Node.js

```bash
npm install -g @devaloop/devalang@latest
```

#### With Rust

```bash
cargo install devalang --version <version>
```

#### Initialize a new project

```bash
devalang init --name my-project --template minimal
```

#### Write your first script

Create a new Devalang file `src/index.deva` in the project directory:

```deva
# src/index.deva

# BPM definition
bpm 125

# Bank picking (make sure you've installed it)
bank devaloop.808 as my808Bank

# Pattern literal without options
pattern kickPattern with my808Bank.kick = "x--- x--- x--- x---"

group myGroup:
    # Rhythmic (each beat playing a kick)
    # on beat:
    #     .my808Bank.kick 1/4

    # Synth definition with ADSR
    let myLead = synth sine {
        attack: 0,
        decay: 100,
        sustain: 100,
        release: 100
    }

    # Global automation
    automate myLead:
        param volume {
            0% = 0.0
            100% = 0.5
        }
        param pitch {
            0% = -12.0
            100% = 12.0
        }

    # Notes in a loop with condition
    for i in [1, 2, 3]:
        if i == 3:
            myLead -> note(C5, { duration: 200 })
            print "Playing note C5 for " + i

    # Pause runtime for 500ms
    sleep 500

    # Note with automation
    myLead -> note(C4, {
        duration: 400,
        velocity: 0.8,
        automate: {
            pan: {
                0%: -1.0,
                100%: 0.0
            }
        }
    })

    # Notes with params
    myLead -> note(G4, { duration: 600, glide: true })
    myLead -> note(B3, { duration: 400, slide: true })

# Spawning the group & the pattern to play them in parallel
spawn myGroup
spawn kickPattern
```

### And the best part ? You can play it directly from the command line:

#### Play the script once

```bash
devalang play
```

#### **LIVE mode** (repeat the playback + watch mode)

```bash
devalang play --repeat
```

### 🎉 You can now hear your Devalang code in action

> For more examples, check out the [examples directory](./examples/)

## ❓ Why Devalang ?

- 🎹 Prototype audio ideas without opening a DAW, even VSCode with our Playground
- 💻 Integrate sound into code-based workflows
- 🎛️ Control audio parameters through readable syntax
- 🧪 Build musical logic with variables and conditions
- 🔄 Create complex patterns with ease

## 🚀 Features

- ⚡ **Fast Build & Hot Reload** — optimized build process for quicker iteration.
- 🎵 **Audio Engine & Real-time runner** — low-latency playback, render-to-file, and a realtime runner used by `devalang play --repeat` for live feedback.
- 🧩 **Language primitives** — synths, notes, ADSR, maps, arrays, loops, conditionals and functions for expressive musical logic.
- 🎛️ **Per-note automation & modulators** — `automate` maps, `$mod.*`, `$easing.*` and `$math.*` helpers for envelopes and LFOs.
- 🧩 **Module system & structured AST** — import/export variables, stable AST output for debugging and tooling.
- 🧰 **Plugins & Addons (WASM-ready)** — install plugins/banks, `@use` directive, and WASM plugin integration so plugins can render or process audio at runtime.
- 📦 **Addon manager & Devaforge** — CLI commands to discover/install banks, plugins and templates; `devaforge` helps create addons.
- ⚙️ **CLI tooling** — `build`, `check`, `play`, `install`, `init`, `discover`, `telemetry` and more with consistent flags (`--watch`, `--debug`, `--compress`).
- 📂 **Project templates & examples** — quick-start templates and many example projects in `examples/`.
- 🧑‍💻 **TypeScript API & WASM distribution** — Node-friendly package with TypeScript bindings and a WASM build for browser/Node usage.
- 🧰 **Editor & formatting support** — VSCode extension and Prettier plugin to edit Devalang with syntax and formatting support.
- 🎵 **Custom samples & banks** — drop samples into `.deva` and reference them from code; banks of sounds for fast composition.
- 🔄 **Looping, grouping & scheduling** — precise beat-tied scheduling primitives for complex rhythmic patterns.

## 📄 Documentation

### [Please refer to the online documentation](https://docs.devalang.com) for detailed information on syntax, features, and usage examples

## 📰 What's new

- **MIDI export**: Added the ability to export MIDI files from Devalang scripts.
- **Synthesizer improvements**: Enhanced the built-in synthesizer with new types and modulation options.
- **Devaforge**: Introduced a new system for creating and managing addons, including a CLI for addon generation.
- **Documentation updates**: Improved documentation for clarity and completeness.
- **Discovering addons**: Introduced a new command to detect addons.
- **Public TypeScript API**: Added a public TypeScript API for easier integration.
- **Improved error messages**: Enhanced error messages for better debugging.
- **Major refactor**: Significant codebase refactor for improved maintainability and performance.
- **Bug fixes**: Various bug fixes and stability improvements.

## 🛡️ License

MIT — see [LICENSE](./LICENSE)

## 🤝 Contributing

Contributions, bug reports and suggestions are welcome !  
Feel free to open an issue or submit a pull request.

For more info, see [docs/CONTRIBUTING.md](./docs/CONTRIBUTING.md).

## 📢 Contact

Feel free to reach out for any inquiries or feedback.

📧 [contact@devaloop.com](mailto:contact@devaloop.com)
