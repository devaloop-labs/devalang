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

![npm](https://img.shields.io/npm/dm/@devaloop/devalang)

## 🎼 Devalang, by **Devaloop Labs**

🎶 Compose music with code — simple, structured, sonic.

Devalang is a tiny domain-specific language (DSL) for music makers, sound designers, and audio hackers.
Compose loops, control samples, and automate parameters — all in clean, readable text.

🦊 Whether you're building a track, shaping textures, or performing live, Devalang helps you think in rhythms. It’s designed to be simple, expressive, and fast — because your ideas shouldn’t wait.

From studio sketches to live sets, Devalang gives you rhythmic control — with the elegance of code.

> 🚧 **v0.0.1-alpha.1 Notice** 🚧
>
> Devalang is still in early development. This version does not yet include **sound rendering**.
>
> You can parse code, generate the AST, and validate syntax — all essential building blocks for the upcoming audio engine.
>
> Currently, only `.kick` is included as a built-in trigger.
> Custom instruments can be defined with `@load`, allowing any sound sample to be triggered with the same syntax.
>
> Currently, Devalang CLI is only available for **Windows**.  
> Linux and macOS binaries will be added in future releases via cross-platform builds.

## 🚀 Features

- 🧩 Module system for importing and exporting variables between files (`@import`, `@export`)
- 📜 Structured AST generation for debugging and future compilation
- 🔢 Basic data types: strings, numbers, booleans, maps, arrays
- ⏱️ `bpm` assignment for setting tempo
- 🧱 `bank` declaration to define the instrument set
- 🔁 Looping system with fixed repetitions (`loop 4:`)
- 🧪 Instruction calls with parameters (e.g. `.kick auto {reverb:10, decay:20}`) for testing pattern syntax
- 📄 `let` assignments for storing reusable values
- 🔄 `@load` assignment to load a sample (.mp3, .wav) to use it as a value
- 🛠️ CLI tools for syntax checking (`check`), AST output (`build`)

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

Usage for development (feel free to change arguments in package.json)

```bash
# For syntax checking test
npm run rust:dev <command>
```

## ❔ Usage

For more examples, see [docs/COMMANDS.md](./docs/COMMANDS.md)

Checking syntax only and output debug files

```bash
devalang check --entry <entry-directory> --output <output-directory>
```

Building output file(s) (AST generation for the moment)

```bash
devalang build --entry <entry-directory> --output <output-directory>
```

## 📄 Syntax example

For more examples, see [docs/SYNTAX.md](./docs/SYNTAX.md)

```deva
# index.deva

@import { globalBpm, globalBank, kickDuration } from "global.deva"

bpm globalBpm
# Will declare the tempo at the globalBpm variable beats per minute

bank globalBank
# Will declare a custom instrument bank using the globalBank variable

loop 5:
    .kick kickDuration {reverb=50, drive=25}
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

- No support yet for Audio Engine
- No support yet for `if`, `else`, `else if` statements
- No support yet for `@group`, `@pattern`, `@function` statements
- Nested loops and conditions may not be fully tested

## 🧪 Roadmap Highlights

For more info, see [docs/ROADMAP.md](./docs/ROADMAP.md)

- ⏳ Audio engine integration
- ⏳ Other statements (e.g `if`, `@group`, ...)
- ⏳ Watch mode for automatic rebuilds

## 🛡️ License

MIT — see [LICENSE](./LICENSE)

## 🤝 Contributing

Contributions, bug reports and suggestions are welcome !  
Feel free to open an issue or submit a pull request.

## 📢 Contact

📧 [contact@devaloop.com](mailto:contact@devaloop.com)
