<div align="center">
    <img src="https://firebasestorage.googleapis.com/v0/b/devaloop-labs.firebasestorage.app/o/devalang-teal-logo.svg?alt=media&token=d2a5705a-1eba-4b49-88e6-895a761fb7f7" alt="Devalang Logo">
</div>

# Roadmap

Devalang is a work in progress. Here’s what we’re planning next:

## Completed

- ✅ **Audio engine**: Integrate the audio engine for sound playback. 
- ✅ **Basic syntax**: Implement the core syntax for Devalang, including data types and basic statements.
- ✅ **Watch mode**: Add a watch mode to automatically rebuild on file changes.
- ✅ **Module system**: Add support for importing and exporting variables between files using `@import` and `@export`.
- ✅ **AST generation**: Implement the Abstract Syntax Tree (AST) generation for debugging and future compilation.
- ✅ **Basic data types**: Support strings, numbers, booleans, maps, and arrays.
- ✅ **BPM assignment**: Implement `bpm` assignment to set the tempo.
- ✅ **Bank declaration**: Add `bank` declaration to define the instrument set.
- ✅ **Looping system**: Implement a looping system with fixed repetitions using `loop 4:`.
- ✅ **Instruction calls**: Add support for instruction calls with parameters (e.g. `.kick auto {reverb:10, decay:20}`).
- ✅ **Let assignments**: Implement `let` assignments for storing reusable values.
- ✅ **Sample loading**: Add `@load` assignment to load samples (.mp3, .wav) for use as values.
- ✅ **WASM support**: Compile Devalang to WebAssembly for use in web applications and other environments.

## Upcoming

- ⏳ **VSCode extension**: Create a VSCode extension for syntax highlighting and code completion.
- ⏳ **Other statements**: Implement `if`, `else`, and other control structures.
- ⏳ **Pattern and group statements**: Add support for `@pattern` and `@group` to organize code.
- ⏳ **Functions**: Add support for defining and calling functions.
- ⏳ **Testing**: Expand test coverage for all features.
