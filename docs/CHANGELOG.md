<div align="center">
    <img src="https://firebasestorage.googleapis.com/v0/b/devaloop-labs.firebasestorage.app/o/devalang-teal-logo.svg?alt=media&token=d2a5705a-1eba-4b49-88e6-895a761fb7f7" alt="Devalang Logo">
</div>

# Changelog

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
