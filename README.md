<div align="center">
    <img src="https://firebasestorage.googleapis.com/v0/b/devaloop-labs.firebasestorage.app/o/devalang-teal-logo.svg?alt=media&token=d2a5705a-1eba-4b49-88e6-895a761fb7f7" alt="Devalang Logo">
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

## 🎼 Devalang, by **Devaloop Labs**

🎶 Compose music with code — structured, expressive, and fast.

Devalang is a tiny domain-specific language (DSL) for music makers, sound designers, and audio hackers.
Compose loops, control samples, render and play audio — all in clean, readable text.

🦊 Whether you're building a track, shaping textures, or performing live, Devalang helps you think in rhythms. It’s designed to be simple, expressive, and fast — because your ideas shouldn’t wait.

From studio sketches to live sets, Devalang gives you rhythmic control — with the elegance of code.

> 🚧 **v0.0.1-alpha.8 Notice** 🚧
>
> NEW: Devalang VSCode extension is now available !
> [Get it here](https://marketplace.visualstudio.com/items?itemName=devaloop.devalang-vscode).
>
> NEW: Devalang supports conditional statements (`if`, `else`, `else if`) for more dynamic compositions !
>
> Currently, Devalang CLI is only available for **Windows**.  
> Linux and macOS binaries will be added in future releases via cross-platform builds.

---

## 📚 Quick Access

- [📖 Documentation](./docs/)
- [💡 Examples](./examples/)
- [🧩 VSCode Extension](https://marketplace.visualstudio.com/items?itemName=devaloop.devalang-vscode)
- [🎨 Prettier Plugin](https://www.npmjs.com/package/@devaloop/prettier-plugin-devalang)
- [🌐 Project Website](https://devalang.com)

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

## 📆 Installation

### For users

> - ⚠️ Requires [Node.js 18+](https://nodejs.org/en/download)

Install the package globally (NPM)

```bash
npm install -g @devaloop/devalang@latest
```

Usage without install (NPX)

```bash
npx @devaloop/devalang@latest
```

### For contributors

> - ⚠️ Requires [Node.js 18+](https://nodejs.org/en/download)
> - ⚠️ Requires [Rust 1.70+](https://www.rust-lang.org/learn/get-started#installing-rust)

```bash
git clone https://github.com/devaloop-labs/devalang.git

cd devalang

npm install
```

Development usage (you can customize arguments in package.json)

```bash
# For syntax checking test
npm run rust:dev:check

# For building test
npm run rust:dev:build
```

## ❔ Usage

NOTE: Commands are available via `devalang` or `npx @devaloop/devalang`.

NOTE: Arguments can be passed to commands using `--<argument>` syntax. You can also use a configuration file to set default values for various settings, making it easier to manage your Devalang project.

NOTE: Some commands require a mandatory `--entry` argument to specify the input folder, and a `--output` argument to specify the output folder. If not specified, they default to `./src` and `./output` respectively.

For more examples, see [docs/COMMANDS.md](./docs/COMMANDS.md)

### Initialize a new project

In the current directory

```bash
devalang init
```

Or use optional arguments to specify a directory name and a template

```bash
devalang init --name <project-name> --template <template-name>
```

### Checking syntax only

```bash
devalang check --watch
```

### Building output files

```bash
devalang build --watch
```

### Playing audio files (once by file change)

```bash
devalang play --watch
```

### Playing audio files (continuous playback, even without file changes)

```bash
devalang play --repeat
```

## ⚙️ Configuration

You can use a configuration file to set default values for various settings, making it easier to manage your Devalang project.

To do this, create a `.devalang` file in the root of your project directory.

See [docs/CONFIG.md](./docs/CONFIG.md) for more information.

## 📄 Syntax example

For more examples, see [docs/SYNTAX.md](./docs/SYNTAX.md)

```deva
# index.deva

@import { globalBpm, globalBank, kickDuration } from "global.deva"

@load "./examples/samples/kick-808.wav" as customKick

bpm globalBpm
# Will declare the tempo at the globalBpm variable beats per minute

bank globalBank
# Will declare a custom instrument bank using the globalBank variable

# Loops

loop 5:
    .customKick kickDuration {reverb=50, drive=25}
    # Will play 5 times a custom sample for 500ms with reverb and overdrive effects

# Groups

group myGroup:
    .customKick kickDuration {reverb=50, drive=25}
    # Will play the same sample in a group, allowing for more complex patterns

# Will be executed line by line (sequentially)
call myGroup

# Will be executed in parallel (concurrently)
# ⚠️ Note: `spawn` runs the entire group in parallel, but the group’s internal logic remains sequential unless it uses `spawn` internally.
# spawn myGroup
```

> 🧠 Note: `call` and `spawn` only work with `group` blocks. They do not apply to individual samples or other statements.

```deva
# variables.deva

let globalBpm = 120
let globalBank = 808
let kickDuration = 500

@export { globalBpm, globalBank, kickDuration }
```

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
