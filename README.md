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

## 🎼 Devalang, by **Devaloop Labs**

🎶 Compose music with code — simple, structured, sonic.

Devalang is a tiny domain-specific language (DSL) for music makers, sound designers, and audio hackers.
Compose loops, control samples, render and play audio — all in clean, readable text.

🦊 Whether you're building a track, shaping textures, or performing live, Devalang helps you think in rhythms. It’s designed to be simple, expressive, and fast — because your ideas shouldn’t wait.

From studio sketches to live sets, Devalang gives you rhythmic control — with the elegance of code.

> 🚧 **v0.0.1-alpha.4 Notice** 🚧
>
> **Audio Engine** is now integrated, enabling audio playback and rendering capabilities.
>
> Currently, Devalang CLI is only available for **Windows**.  
> Linux and macOS binaries will be added in future releases via cross-platform builds.

---

## 📚 Quick Access

- [📖 Documentation](./docs/)
- [💡 Examples](./examples/)
- [🌐 Project Website](https://devalang.com)

## 🚀 Features

- 🎵 **Audio Engine**: Integrated audio playback and rendering
- 🧩 **Module system** for importing and exporting variables between files
- 📜 **Structured AST** generation for debugging and future compilation
- 🔢 **Basic data types**: strings, numbers, booleans, maps, arrays
- 👁️ **Watch mode** for `build`, `check` and `play` commands
- 📂 **Project templates** for quick setup

## 📆 Installation

### For users

> - ⚠️ Requires [Node.js 18+](https://nodejs.org/en/download)

Install the package globally (NPM)

```bash
npm install -g @devaloop/devalang
```

Usage without install (NPX)

```bash
npx @devaloop/devalang <command>
```

### For contributors

> - ⚠️ Requires [Node.js 18+](https://nodejs.org/en/download)
> - ⚠️ Requires [Rust 1.70+](https://www.rust-lang.org/learn/get-started#installing-rust)

```bash
> git clone https://github.com/devaloop-labs/devalang.git
> cd devalang
> npm install
> cargo install --path .
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

loop 5:
    .customKick kickDuration {reverb=50, drive=25}
    # Will play 5 times a kick for the duration of the kickDuration variable with reverb and drive effects
```

```deva
# global.deva

let globalBpm = 120
let globalBank = 808
let kickDuration = 500

@export { globalBpm, globalBank, kickDuration }
```

## 🧯 Known issues

- No support yet for `if`, `else`, `else if` statements
- No support yet for `@group`, `@pattern`, `@function` statements
- No support yet for cross-platform builds (Linux, macOS)

## 🧪 Roadmap Highlights

For more info, see [docs/ROADMAP.md](./docs/ROADMAP.md)

- ⏳ Other statements (e.g `if`, `@group`, ...)
- ⏳ Cross-platform support (Linux, macOS)
- ⏳ More built-in instruments (e.g. snare, hi-hat, etc.)

## 🛡️ License

MIT — see [LICENSE](./LICENSE)

## 🤝 Contributing

Contributions, bug reports and suggestions are welcome !  
Feel free to open an issue or submit a pull request.

## 📢 Contact

📧 [contact@devaloop.com](mailto:contact@devaloop.com)
