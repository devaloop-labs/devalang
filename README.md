<div align="center">
    <img src="https://devalang.com/images/devalang-logo-cyan.svg" alt="Devalang Logo" width="300" />
</div>

![Rust](https://img.shields.io/badge/Made%20with-Rust-orange?logo=rust)
![TypeScript](https://img.shields.io/badge/Built%20with-TypeScript-blue?logo=typescript)
![Node.js](https://img.shields.io/badge/Node.js-18%2B-brightgreen?logo=node.js)

![Project Status](https://img.shields.io/badge/status-alpha-red)
![Version](https://img.shields.io/badge/version-0.0.1-blue)
![License: MIT](https://img.shields.io/badge/license-MIT-green)
![Platform](https://img.shields.io/badge/platform-Windows-blue)

![npm](https://img.shields.io/npm/dt/@devaloop/devalang)
![crates](https://img.shields.io/crates/d/devalang)

[![VSCode Extension](https://img.shields.io/visual-studio-marketplace/v/devaloop.devalang-vscode?label=VSCode%20Extension)](https://marketplace.visualstudio.com/items?itemName=devaloop.devalang-vscode)

# 🎶 Devalang

Compose music with code — structured, expressive, and fast.

Devalang is a tiny domain-specific language (DSL) for music makers, sound designers, and audio hackers.
Compose loops, control samples, render and play audio — all in clean, readable text.

🦊 Whether you're building a track, shaping textures, or performing live, Devalang helps you think in rhythms. It’s designed to be simple, expressive, and fast — because your ideas shouldn’t wait.

From studio sketches to live sets, Devalang gives you rhythmic control — with the elegance of code.

> 🚧 **v0.0.1-alpha.11 Notice** 🚧
>
> NEW: Devalang is available in your browser at [playground.devalang.com](https://playground.devalang.com) !
>
> NEW: Online documentation is now available at [docs.devalang.com](https://docs.devalang.com)
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

### You can also have a look at the [Playground](https://playground.devalang.com) to try Devalang directly in your browser

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

group myLead:
    let mySynth = synth sine

    mySynth -> note(C4, { duration: 400 })
    mySynth -> note(G4, { duration: 600 })

# Play the lead

call myLead
```

And the best part ? You can play it directly from the command line:

```bash
# Play the Devalang file
devalang play

# Play the Devalang file with watch mode
devalang play --watch

# LIVE mode (repeat the playback + watch mode)
devalang play --repeat
```

### 🎉 You can now hear your Devalang code in action!

> For more examples, check out the [examples directory](./examples/).

## ❓ Why Devalang?

- 🎹 Prototype audio ideas without opening a DAW
- 💻 Integrate sound into code-based workflows
- 🎛️ Control audio parameters through readable syntax
- 🧪 Build musical logic with variables and conditions

> Producer, coder, or both — Devalang gives you musical structure, instantly.

## 🚀 Features

- 🎵 **Audio Engine**: Integrated audio playback and rendering
- 🧩 **Module system** for importing and exporting variables between files
- 📜 **Structured AST** generation for debugging and future compilation
- 🔢 **Basic data types**: strings, numbers, booleans, maps, arrays
- 👁️ **Watch mode** for `build`, `check` and `play` commands
- 📂 **Project templates** for quick setup
- 🎛️ **Custom samples**: easily load and trigger your own audio files
- 🔄 **Looping and grouping**: create complex patterns with ease

## 📄 Documentation

### Please refer to the [online documentation](https://docs.devalang.com) for detailed information on syntax, features, and usage examples.

## 📜 Changelog Highlights

For a complete list of changes, see [docs/CHANGELOG.md](./docs/CHANGELOG.md)

- Implemented beat durations in `triggers` and `arrow_calls` statements
- Implemented `bank` resolver to resolve banks of sounds in the code
  - Support for namespaced banks of sounds (e.g. `.808.myTrigger`)
- Implemented multiple commands for `bank` management
  - `bank list`, `bank available`, `bank info <bank_name>`, `bank remove <bank_name>`, `bank update`, `bank update <bank_name>`
- Implemented `install` command to install banks of sounds
  - `install bank <bank_name>`

## 🧯 Known issues

- No smart modules yet, all groups, variables, and samples must be explicitly imported where used
- No support yet for `pattern`, `function`, ... statements
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

## 📢 Contact

📧 [contact@devaloop.com](mailto:contact@devaloop.com)
