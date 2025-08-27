<div align="center">
    <img src="https://devalang.com/images/devalang-logo.svg" alt="Devalang Logo" width="300" />
</div>

![Rust](https://img.shields.io/badge/Made%20with-Rust-orange?logo=rust)
![TypeScript](https://img.shields.io/badge/Built%20with-TypeScript-blue?logo=typescript)
![Node.js](https://img.shields.io/badge/Node.js-18%2B-brightgreen?logo=node.js)

![Project Status](https://img.shields.io/badge/status-alpha-red)
![Version](https://img.shields.io/npm/v/@devaloop/devalang)
![License: MIT](https://img.shields.io/badge/license-MIT-green)
![Platform](https://img.shields.io/badge/platform-Windows-blue)

![npm](https://img.shields.io/npm/dt/@devaloop/devalang)
![crates](https://img.shields.io/crates/d/devalang)

[![VSCode Extension](https://img.shields.io/visual-studio-marketplace/v/devaloop.devalang-vscode?label=VSCode%20Extension)](https://marketplace.visualstudio.com/items?itemName=devaloop.devalang-vscode)

# 🦊 Devalang (CORE) — Compose music with code

Devalang is a tiny domain-specific language (DSL) for music makers, sound designers, and audio hackers.
Compose loops, control samples, render and play audio — all in clean, readable text.

Whether you're building a track, shaping textures, or performing live, Devalang helps you think in rhythms. It’s designed to be simple, expressive, and fast — because your ideas shouldn’t wait.

From studio sketches to live sets, Devalang gives you rhythmic control — with the elegance of code.

> 🚧 Alpha Notice 🚧
>
> Includes synthesis, playback, and rendering features, but is still in early development.
>
> Currently, Devalang CLI is only available for **Windows**.
> Linux and macOS binaries will be added in future releases via cross-platform builds.

## 📚 Quick Access

- [▶️ Playground](https://playground.devalang.com)
- [📖 Documentation](https://docs.devalang.com)
- [🧩 VSCode Extension](https://marketplace.visualstudio.com/items?itemName=devaloop.devalang-vscode)
- [🎨 Prettier Plugin](https://www.npmjs.com/package/@devaloop/prettier-plugin-devalang)
- [📜 Changelog](./docs/CHANGELOG.md)
- [💡 Examples](./examples/)
- [🌐 Project Website](https://devalang.com)
- [📦 Devalang CLI on npm](https://www.npmjs.com/package/@devaloop/devalang)

## ⏱️ Try it now !

### Try Devalang in your browser

> Have a look at the [Playground](https://playground.devalang.com) to try Devalang directly in your browser

### Try Devalang CLI

```bash
# Install Devalang CLI globally
npm install -g @devaloop/devalang

# Create a new Devalang project
devalang init --name my-project --template minimal
cd my-project
```

Create a new Devalang file `src/index.deva` in the project directory:

```deva
# src/index.deva

group main:
    let lead = synth sine {
        attack: 0,
        decay: 100,
        sustain: 100,
        release: 100
    }

    # Global automation for this synth (applies to subsequent notes)
    automate lead:
        param volume {
            0% = 0.0
            100% = 1.0
        }
        param pan {
            0% = -1.0
            100% = 1.0
        }
        param pitch {
            0% = -12.0
            100% = 12.0
        }

    lead -> note(C4, {
        duration: 400,
        velocity: 0.8,
        automate: { pan: { 0%: -1.0, 100%: 0.0 } }
    })

    lead -> note(E4, { duration: 400 })
    lead -> note(G4, { duration: 600, glide: true, target_freq: 659.25 })
    lead -> note(B3, { duration: 400, slide: true, target_amp: 0.3 })

    for i in [1, 2, 3]:
        lead -> note(C5, { duration: 200 })

# Play the lead

call main
```

### And the best part ? You can play it directly from the command line:

```bash
# Play the Devalang file
devalang play

# Play the Devalang file with watch mode
devalang play --watch

# LIVE mode (repeat the playback + watch mode)
devalang play --repeat
```

### 🎉 You can now hear your Devalang code in action

> For more examples, check out the [examples directory](./examples/)

## ❓ Why Devalang ?

- 🎹 Prototype audio ideas without opening a DAW, even VSCode
- 💻 Integrate sound into code-based workflows
- 🎛️ Control audio parameters through readable syntax
- 🧪 Build musical logic with variables and conditions
- 🔄 Create complex patterns with ease

## 🚀 Features

- 🎵 **Audio Engine**: Integrated audio playback and rendering
- 🧩 **Module system** for importing and exporting variables between files
- 📦 **Addon manager** for managing external banks, plugins and more
- 📜 **Structured AST** generation for debugging and future compilation
- 🔢 **Basic data types**: strings, numbers, booleans, maps, arrays
- 👁️ **Watch mode** for `build`, `check` and `play` commands
- 📂 **Project templates** for quick setup
- 🎛️ **Custom samples**: easily load and trigger your own audio files
- 🔄 **Looping and grouping**: create complex patterns with ease

## 📄 Documentation

### Please refer to the [online documentation](https://docs.devalang.com) for detailed information on syntax, features, and usage examples

## 🧯 Known issues

- No smart modules yet, all groups, variables, and samples must be explicitly imported where used
- No support yet for cross-platform builds (Linux, macOS)

## 🧪 Roadmap Highlights

For more info, see [docs/ROADMAP.md](./docs/ROADMAP.md)

- ⏳ Other statements (e.g `function`, `pattern`, ...)
- ⏳ Cross-platform support (Linux, macOS)
- ⏳ More built-in instruments (e.g. snare, hi-hat, etc.)

## 🛡️ License

MIT — see [LICENSE](./LICENSE)

## 🤝 Contributing

Contributions, bug reports and suggestions are welcome !  
Feel free to open an issue or submit a pull request.

For more info, see [docs/CONTRIBUTING.md](./docs/CONTRIBUTING.md).

## 📢 Contact

Feel free to reach out for any inquiries or feedback.

📧 [contact@devaloop.com](mailto:contact@devaloop.com)
