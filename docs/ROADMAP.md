<div align="center">
    <img src="https://devalang.com/images/devalang-logo.svg" alt="Devalang Logo" width="300" />
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
- ✅ **VSCode extension**: Create a VSCode extension for syntax highlighting and code completion.

## Upcoming

- ⏳ **Smart modules**: Let Devalang detect and use groups, samples, and variables without needing to import them manually.
- ⏳ **Other statements**: Implement `pattern`, `function`, and other control structures.
- ⏳ **Testing**: Expand test coverage for all features.