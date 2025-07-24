<div align="center">
    <img src="https://devalang.com/images/devalang-logo-cyan.svg" alt="Devalang Logo" width="300" />
</div>

# Changelog

## Version 0.0.1-alpha.13 (2025-07-24)

### 🧩 Language Features

- Added support for `fn` directive to define functions in Devalang.
  - Example: `fn myFunction(param1, param2):`
- Possibility to use triggers inside `let` statements, allowing for more dynamic sound manipulation.
  - Example: `let myTrigger = .myTrigger auto { reverb: 0.5, pitch: 1.2 }`

### 🧠 Core Engine

- Patched `trigger`, `call`, and `spawn`, `renderer` to handle correct cursor time in the audio interpreter.
- Refactored audio engine and interpreter to handle correct timing and execution while using `loop`, `call`, and `spawn` statements.
- Refactored `trigger` effects to apply more effects to triggers.
  - Example: `.myTrigger auto { reverb: 0.25, pitch: 0.75, gain: 0.8 }`
- Refactored `preprocessor` to handle correct namespaced banks of sounds and triggers.

## Version 0.0.1-alpha.12 (2025-07-21)

### 🧩 Language Features

- Implemented `trigger` effects to apply effects to triggers, allowing for more dynamic sound manipulation.
  - Example: `.myTrigger auto { reverb: 1.0, pitch: 1.5, gain: 0.8 }`

### 🧠 Core Engine

- Moved `utils::installer` to `installer::utils` to better organize the project structure.
- Set CLI dependencies as optional in `Cargo.toml` to allow for a cleaner build without CLI features.
- Patched `@load` relative path resolution to ensure correct loading of external resources.
- Patched `trigger` statement that was not correctly parsed when using namespaced banks of sounds.

### 🧩 Web Assembly

- Patched `lib.rs` dependencies to ensure compatibility with the latest Rust and WASM standards.

## Version 0.0.1-alpha.11 (2025-07-20)

### 📖 Documentation

- Removed old documentation, please refer to the [new documentation website](https://docs.devalang.com) for the latest information.

### ✨ Syntax

- Added namespaced banks of sounds, allowing for better organization and management of sound banks.
  - Example: `.808.myTrigger` to access a specific trigger in the `808` bank.
- Added support for beat durations in `triggers` statements, allowing for more precise timing control.
  - Example: `.myTrigger 1/4 { ... }` to trigger the sound every quarter beat.
- Added support for beat durations in `arrow_calls` statements, allowing for more precise timing control.
  - Example: `mySynth -> note(C4, { duration: 1/8 })` to play a note for an eighth beat.

### 🧠 Core Engine

- Implemented `bank` resolver to resolve banks of sounds in the code.
  - Example: `bank 808` will resolve to a bank of sounds named `808` if exists (check bank available command).

### 🧰 Commands

> Use the `bank 808` statement to access the default sounds and triggers !
> Then you can use `808.myTrigger` to access a specific trigger in the `808` bank.

- Added `bank` command to manage banks of sounds.

  - `bank list` to list installed banks of sounds.
  - `bank available` to list available banks of sounds for installation.
  - `bank info <bank_name>` to show information about a specific bank.
  - `bank remove <bank_name>` to remove a bank.
  - `bank update` to update all banks of sounds.
  - `bank update <bank_name>` to update a specific bank.

- Added `install` command to install banks of sounds.
  - `install bank <bank_name>` to install a specific bank of sounds.

### 🧪 Experimental

- Introduced lazy loading and namespace-based resolution of installed sound banks.

## Version 0.0.1-alpha.10 (2025-07-19)

### 📖 Documentation

- Updated [new documentation website](https://docs.devalang.com) with new features and examples.

### 🧠 Core Engine

- Patched `call`, `spawn` to handle correct cursor time.
- Patched `advance_char` to handle correct indentation and dedentation.
- Patched `bank` resolver to handle numbers in bank names.
- Patched `spawn` calls that was not calling a variable.

### 🧩 Web Assembly

- Added `load_wasm_module` function to the WASM module to load Devalang modules dynamically.
- Added `render_audio` function to the WASM module to render audio files.

## Version 0.0.1-alpha.9 (2025-07-14)

### ✨ Syntax

- Added support for `synth` directives to define synthesizer sounds directly in code  
  → Example:

  ```deva
  let mySynth = synth sine
  ```

### 🧠 Core Engine

- ✅ **Major refactor** of the resolution layer:

  - `condition`, `loop`, `group`, `trigger`, `call`, `spawn`, and `driver` were fully rewritten
  - Ensures **correct variable scoping**, **cross-module references**, and **imported symbol resolution**

- ✅ Audio interpreter updated:
  - `call` and `spawn` now execute correctly in parallel, preserving **temporal alignment** across groups

### 🧩 Language Features

- Improved `@import` behavior:
  - Modules can now be imported using **relative paths only**
  - No need for absolute or root-based imports

## Version 0.0.1-alpha.8 (2025-07-12)

### Syntax

- Implemented `if` directive to conditionally execute blocks of code.
- Implemented `else` directive to provide an alternative block of code when the `if` condition is not met.
- Implemented `else if` directive to provide additional conditions for the `if` directive.

### Core Components

- Implemented evaluator for audio statements, to execute conditional statements.
- Fixed `group` resolution and export issues.
- Implemented `Global Store` debugger to inspect variables by module for build command.
- Organized `TokenKind` and `StatementKind` enums for better clarity and maintainability.
- Refactored audio interpreter to handle the new syntax and directives.
- Refactored lexer to handle new directives and improve tokenization.
- Refactored parser to handle new directives and improve parsing logic.
- Added support for `call` and `spawn` execution of imported groups.
- Enforced scoped resolution of groups in `spawn` and `call` (must be imported in current module).

## Version 0.0.1-alpha.7 (2025-07-11)

## Examples

- Added examples in `examples` folder (group, loop, variables, index).

## Structure

- Moved `rust/audio` folder to `rust/core/audio` to better organize the project structure.

### Core Components

- Implemented `group` directive to define groups of sounds.
- Implemented `call` directive to call a group of sounds.
- Implemented `spawn` directive to spawn a group of sounds in parallel.
- Implemented `sleep` directive to pause execution for a specified duration.
- Patched line and column tracking in the lexer to ensure correct indentation handling.
- Patched string lexing advancing to handle first character correctly.

## Version 0.0.1-alpha.5 (2025-07-05)

### Syntax

- Fixed block parsing issues caused by missing or incorrect `Indent` / `Dedent` token detection.
- Indentation handling now triggers correctly at each newline.
- Improved reliability of nested blocks (e.g., inside `loop`) with consistent `Dedent` termination.

### Core Components

- Added full **WebAssembly (WASM)** support — Devalang can now be compiled for browser or Node.js environments.
- Prepared the ground for future IDE integrations (e.g., VSCode extension) by stabilizing core syntax parsing.

## Version 0.0.1-alpha.4 (2025-07-03)

### Audio Engine

- Integrated Audio Engine to handle audio playback and rendering.
- Implemented Audio Player to play audio files.
- Added support for audio playback with the `play` command.

### Commands

- Implemented `play` command to play Devalang files.

  - Added `--watch` option to watch for changes in files and automatically rebuild and play them. (once)
  - Added `--repeat` option to repeat the playback of the audio file. (infinite)

  Note : You cannot use `--watch` and `--repeat` options together. Use `--repeat` instead.

## Version 0.0.1-alpha.3 (2025-07-01)

- /!\ Major refactor of the project structure and module system /!\
- Refactored module system to support multiple modules and submodules.
- Patched all directives to be compatible with the new project structure.
- Prepared for the upcoming audio engine integration and sound rendering capabilities.
- Updated documentation to reflect the new project structure and features.

## Version 0.0.1-alpha.2 (2025-06-26)

### Commands

- Implemented `init` command to initialize a new Devalang project.
- Implemented `template` command to manage templates.
  - Added `list` subcommand to list available templates.
  - Added `info` subcommand to show information about a specific template.
- Implemented `watch` subcommand for the `build` and `check` command to watch for changes in files and automatically rebuild or check them.

### Core Components

- Implemented Config manager to handle configuration files.
  - Added support for `.devalang` configuration file as a TOML file.
- Implemented File System watcher to monitor file changes.
- Implemented Template manager to handle templates and their metadata.

### Syntax

- Added support for built-in triggers for `.snare`, `.hihat`, `.clap`, `.tom`, `.crash`, `.ride`, `.synth`, `.bass`, and `.pad`.

## Version 0.0.1-alpha.1 (2025-06-25)

### Syntax

- Added support for `@import` directive to import other Devalang files.
- Added support for `@export` directive to export variables and functions.
- Added support for `@load` directive to load external resources.
- Added support for `bpm` directive to set the beats per minute.
- Added support for `bank` directive to define a bank of sounds.
- Added support for `loop` directive to define loops in the code.

### Commands

- Implemented `check` command to check the syntax of Devalang files.
- Implemented `build` command to build the Abstract Syntax Tree (AST) of Devalang files.

### Core Components

- Implemented Lexer to tokenize Devalang source code.
- Implemented Parser to parse the tokens and build the AST.
- Implemented Preprocessor to handle directives and preprocess the source code.
- Implemented Debugger to debug Devalang code.
- Implemented Builder to build the final output from the AST.
