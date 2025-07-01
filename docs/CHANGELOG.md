<div align="center">
    <img src="https://firebasestorage.googleapis.com/v0/b/devaloop-labs.firebasestorage.app/o/devalang-teal-logo.svg?alt=media&token=d2a5705a-1eba-4b49-88e6-895a761fb7f7" alt="Devalang Logo">
</div>

# Changelog

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
