<div align="center">
    <img src="https://devalang.com/images/devalang-logo-min.png" alt="Devalang Logo" width="100" />
</div>

# Changelog

## Version 0.0.1-beta.2 (2025-09-07)

### 🛠️ MIDI & format

- Added support for `mid` export in `build` command.
  - Example: `devalang build --output-format mid,wav`
- Added support for `wav16` and `wav32` audio formats in `play` command.
  - Example: `devalang play --audio-format <wav16 | wav32>`
- Added support for `sample_rate` parameter in `play` and `build` commands.
  - Example: `devalang play --sample-rate <44100 | 48000 | 96000>`
  - Example: `devalang build --sample-rate <44100 | 48000 | 96000>`

### ✨ Language Features

- `synth` enhancements:
  - Types:
    - Added `pad` type for lush, evolving sounds.
    - Added `pluck` type for sharp, percussive tones.
    - Added `arp` type for arpeggiated sequences.
    - Added `sub` type for deep bass sounds.
  - LFOs:
    - Added `lfo` parameter to `synth` for low-frequency oscillation modulation.
    - LFO supports `rate`, `depth`, and `target` parameters.
  - Filters:
      - Added `filters` parameter to `synth` for applying filters.
- `arrow_call` enhancements:
  - Added support for `chord` method to play multiple notes simultaneously.

### 🧠 Architecture & Refactor

- Modularized some core components to improve maintainability and readability.

### 📚 Examples

- Added `examples/filter.deva` to showcase filter usage in synths.
- Added `examples/lfo.deva` to illustrate LFO modulation in synths.
- Added `examples/synth_types.deva` to demonstrate various synth types.

### 📦 WASM

- Added `collect_playhead_events` function to the WASM module to collect playhead events during audio rendering.

## Version 0.0.1-beta.1 (2025-09-02)

> First beta of Devalang 0.0.1. Focus on stability, language surface freeze, and developer experience. No breaking changes expected compared to alpha.18; experimental features are gated and may change.
>
> NOTE: this beta release does not include pre-built WASM/Node bindings in the npm package. If you need WASM bindings, build them locally in the `rust/devalang` directory with `wasm-pack build --target nodejs` and copy the generated `pkg` into `out-tsc/pkg/`, or wait for a future release that bundles the artifacts.

### ✨ Language Features

- Language surface freeze for beta: core syntax and semantics stabilized; future breaking changes will follow a deprecation policy.
- `pattern` enhancements: improved scheduling accuracy and step handling; supports accents and swing via step markers; clearer error messages for malformed patterns.
- Better diagnostics across the language: consistent error formatting with source spans and error codes.

### 🧠 Core Engine

- High-precision scheduler: tighter timing guarantees and improved alignment with BPM; more reliable loop execution and periodic handlers.
- Audio quality: automatic micro-fades on start/end to reduce clicks; improved handling of tails and voice polyphony.
- Sample engine: safer stereo/mono handling and level preservation; fewer edge-case distortions on rapid retriggers.

### 🧩 Parser & Lexer

- Span-rich errors across maps/arrays and complex expressions; clearer messages and recovery where possible.
- Lexer driver matured to cleanly separate file resolution from tokenization; fewer panics and better diagnostics.

### 🔁 Preprocessor & Resolution

- Deterministic include/resolve order; variable lookup consistently walks parent scopes.
- Pattern resolver integration hardened to ensure `call`/`spawn` of patterns behave like functions/groups.

### 🛠️ CLI & Telemetry

- `build`, `check`, `play`: consistent exit codes and non-watch behavior; clearer surfacing of errors.
- `install`/`discover`: authentication flow improved; better JSON/API error reporting; integrity checks on downloaded artifacts.
- Telemetry: stable anonymous machine UUID; records CLI version/OS/args with opt-out capability; ensures `.deva` directory exists.

### 📦 Packaging (WASM/Node/Rust)

- WASM/TypeScript distribution: safer postinstall with guarded downloads; robust `.d.ts` types and ESM/CJS compatibility.
- Rust crates: metadata completed and internal versions pinned; native-only features kept optional to reduce footprint.

### 🧪 Tests & CI

- Added unit tests for parser, pattern scheduling, and selected core utilities; initial wasm smoke tests.
- CI matrix across Linux/macOS/Windows and Node/Rust versions; release pipeline hardened.

## Version 0.0.1-alpha.18 (2025-09-02)

### ✨ Language Features

- New `pattern` statement to define rhythmic patterns with an optional target entity.
  - Example: `pattern kickPattern with my808.kick = "x--- x--- x--- x---"`
- Patterns can be invoked with `call` or `spawn` just like functions or groups.

### 🧠 Core Engine

- Pattern playback: schedules steps across one bar (4 beats), computing per-step duration and triggering the target on non-rest characters.
- ADSR envelope: improved interpolation at segment boundaries to avoid clicks and handle 0/1-sample edge cases.
- Sample engine: robust stereo-to-mono mixdown with RMS-preserving scaling; applies a tiny automatic fade (~1 ms) when samples start/end abruptly to reduce clicks.

### 🧩 Parser & Lexer

- Added `Pattern` token and parser handler; supports `pattern <name> [with <bank.trigger>] = "..."`.
- Introduced a dedicated lexer driver (`rust/core/lexer/driver.rs`) to separate file resolution from tokenization.
- Map/array parsing now logs structured errors via the shared logger instead of printing to stdout.

### 🔁 Preprocessor & Resolution

- Pattern resolver stores definitions in the variable table, enabling later `call`/`spawn` usage.
- Variable lookup now walks parent scopes, fixing missed resolutions for outer-scope identifiers.

### 🛠️ CLI & Telemetry

- `build`: non-watch mode now executes and surfaces errors correctly.
- `install`: requires authentication and reports API/JSON errors with clear messages.
- Telemetry: generates a stable UUID when missing; consistently records CLI version, OS, and args.
- Ensures the `.deva` directory exists at startup.

### 📚 Examples

- Added `examples/pattern.deva`; updated `examples/index.deva` to demonstrate `pattern` and `spawn`.

### 📦 Packaging

- Added crate metadata (description, license, authors) and pinned internal versions for `devalang_types` and `devalang_utils`.

### 🐛 Fixes & Stability

- Safer `$math` parsing with diagnostics for malformed calls and argument evaluation failures.
- Minor parser fixes (loop body collection, clearer error messages) and logging cleanups across modules.

## Version 0.0.1-alpha.17 (2025-08-30)

### ✨ Addons

- Discovering addons: use the CLI to discover plugins or banks : place compiled addons (.devabank, .devaplugin) anywhere into your `.deva` folder, then run:

  ```bash
  devalang discover
  ```

- Installing external addons: use the CLI to install addons:

  ```bash
  devalang install <plugin | bank> user.myPlugin
  ```

- Plugin usage: you can now reference installed plugins directly from Devalang source
  using the `@use` directive and an optional alias. Examples:

  - `@use user.myPlugin` — exposes the plugin under its default name
  - `@use user.myPlugin as myAlias` — expose the plugin as `myAlias` for shorter calls

### 🧩 Packaging & TypeScript

- Improved packaging and postinstall logic for the TypeScript/WASM distribution; the
  Node package now avoids hard-failing when optional native binaries are not available.
- `out-tsc` postinstall now performs guarded downloads and better logging to help consumers diagnose installation issues on CI or constrained environments.

### 🧠 Architecture & Refactor

- Introduced a shared `devalang_types` crate to centralize project types and config structures — this simplifies cross-crate typing between the CLI, core, utils and WASM artifacts.
- Split utilities into a reusable `devalang_utils` crate (logger, watcher, spinner, file helpers, telemetry) and moved several helpers (path resolution, file copying, safe archive extraction) to that crate.
- Modularized CLI features, clean separation between build/process, realtime runner, IO helpers and stats collection.

### 🐛 Bug fixes & stability

- Multiple robustness fixes across the parser, preprocessor and audio engine:
  - dotted identifiers and synth/provider resolution improvements,
  - safer path resolution for `.deva` resources (banks, plugins, presets),
  - improved error collection and logging with annotated stacks for better debug output.
- Audio runtime: better BPM/duration estimation and loop handling in the realtime runner to avoid premature termination of periodic handlers while loops execute.

### 🧪 Tests & CI

- Added test scaffolding and types to centralize cross-crate tests (test harness and lightweight types prepared in `devalang_types`); this enables adding unit tests incrementally without duplicating type definitions. (Full test coverage is ongoing.)

## Version 0.0.1-alpha.16-hotfix.2 (2025-08-29)

### 🌎 Ecosystem

- Published `@devaloop/devaforge` on npm. A tool for creating and managing Devalang addons.

### 🔎 Telemetry

- Patched first usage and user configuration.

## Version 0.0.1-alpha.16-hotfix.1 (2025-08-29)

### 🌎 Ecosystem

- Fixed Github Actions to build and release binaries for multiple platforms.
- Fixed `postinstall` script to launch properly postinstall.js when installing core package

## Version 0.0.1-alpha.16 (2025-08-28)

### 🌎 Ecosystem

- Added Github Actions to build and release binaries for multiple platforms.

### 🧩 Language Features

- `bank` handler: add `as <alias>` support and robust parsing of `author.name` and string names.
  - Example: `bank user.808 as my808`, `bank user.myBank as myBank`
- `plugin` handler: initial implementation with basic support for loading and resolving plugins.
  - Example: `@use user.myPlugin`, `@use user.myPlugin as myAlias`
- `on` event handler: implemented event trigger resolution and context management.
  - Example: `on beat: ...`, `on bar: ...`, `on custom: ...`
- `emit` event handler: initial implementation for emitting events.
  - Example: `emit beat`, `emit custom { value: 42 }`
- `print`: add JS-like string concatenation with `+` between strings, variables, numbers, and `$env`/`$math` expressions.
  - Examples: `print "looping " + i`, `print "bpm=" + $env.bpm`, `print "sin=" + $math.sin(0.5)`

### 🧠 Core Engine

- Cleanest error handling for unknown triggers (module:line:column), no implicit file search.
- Real-time runner (play): loops are paced at 1 iteration per beat on the same thread.
  - Periodic events (`on beat`, `on $beat`, `on bar`, `on $bar`) are suspended while a loop is running to avoid interleaving.
  - Loops stop strictly at the end of their block (dedent / end-of-line).
  - Duration estimation improved by accounting for loop iteration counts to keep the runner alive as needed.

### 🔎 Telemetry (stats)

- Added basic telemetry support for tracking module loading and resolution times.
  - To enable telemetry, execute `devalang telemetry enable`
  - To disable telemetry, execute `devalang telemetry disable`

## Version 0.0.1-alpha.15 (2025-08-27)

### ✨ Language Features

- Added `automate` statement to schedule parameter automation (e.g. volume, pan, pitch) over time
  - Supports per-note automation via a `automate` map on note calls (e.g. `note(C4, { automate: { volume: { 0%: 0.0, 100%: 1.0 } } })`)
- Added `print` statement to ease debugging at runtime
- Added special variables and functions usable in expressions:
  - `$env.bpm`, `$env.beat`
  - `$math.sin(expr)`, `$math.cos(expr)`
  - `$env.position` (alias of beat), `$env.seed` (global session seed for deterministic randomness)
  - `$math.random(seed?)`, `$math.lerp(a, b, t)`
  - `$easing.*` functions for shaping values in [0,1]:
    - `linear`, `easeIn/Out/InOutQuad`, `easeIn/Out/InOutCubic`, `easeIn/Out/InOutQuart`,
      `easeIn/Out/InOutExpo`, `easeIn/Out/InOutBack`, `easeIn/Out/InOutElastic`, `easeIn/Out/InOutBounce`
  - `$mod.*` modulators for time-based control:
    - `lfo.sine(ratePerBeat)`, `lfo.tri(ratePerBeat)`, `envelope(attack, decay, sustain, release, t)`
- Added basic `for` loops and array literals
  - Example: `for i in [1, 2, 3]: print i`

### 🧠 Core Engine

- Implemented runtime automation in the audio renderer with linear envelope interpolation
- Per-note automation supported (volume, pan, pitch) and evaluated during rendering
- Fixed evaluator recursion guard and improved `$math.*` expression handling (prevents stack overflows)
- Minor ADSR defaults polish: ensure `sustain` defaults to `1.0`
- Evaluator now supports `$mod.*` and `$easing.*` calls (evaluated before `$math.*`) for richer automation
- Modularized `AudioEngine::insert_note` into small helpers (oscillator, ADSR computation, pan gains, envelope evaluation, stereo mix)
  - Reused helpers in `pad_samples` to reduce duplication
- Moved special variables/functions to a dedicated module: `core::audio::special`, and refactored the evaluator to use it
- Continued borrow-friendly refactors to avoid unnecessary clones and improve readability

### 🧩 Parser / Preprocessor

- Parser upgrades for operators `+ - * /`, parentheses and brackets
- Improved arrow-call parsing and map handling for multi-line values
- Resolvers: refined `call`/`spawn` resolution (better error messages with stack traces)

### 🧱 Architecture / Refactor

- Modularized audio interpreter (split by statement type); clearer responsibilities
- Reduced allocations by passing slices/borrows instead of cloning large structures
- Removed dead code and unused params across resolvers, handlers, and interpreter modules

### 🧰 Tooling / Build

- Resolved binary/lib artifact collision by renaming the internal library crate to `devalang_core`
- Warning sweep: build now compiles cleanly without Rust warnings (and fewer Clippy lints)
- Moved error collection helpers into a dedicated `utils::error` module

### 🐛 Fixes & Stability

- Prevent infinite recursion during numeric expression evaluation
- Stabilized renderer and interpreter timing when combining `loop`, `call`, and `spawn`

### ⚠️ Breaking changes

- Internal crate rename to `devalang_core` (no change to the CLI or WASM package names)

## Version 0.0.1-alpha.14 (2025-08-24)

### 🌎 Ecosystem

- Deployed "SSO" (Single Sign-On) for user authentication. [(https://sso.devalang.com)](https://sso.devalang.com) when using `devalang login`.

### 🧩 Language Features

- Added support for ADSR envelopes in synthesizers.
  - Example: `let mySynth = synth sine { attack: 0, decay: 50, sustain: 100, release: 50 }`
- Added support for `note` parameters in synthesizers.
  - Example: `mySynth -> note(C4, { duration: 500, velocity: 0.8, glide: true, slide: false })`

### 🧠 Core Engine

- Patched banks resolution with improved namespace handling. (declaring `bank <bank_author>.<bank_name>` and using `.<bank_name>.<bank_trigger>`)
- Patched `arrow_call` to correctly handle argument parsing and improve error reporting.
  - Implemented multi-line argument parsing for `arrow_call`.
  - Patched execution of `arrow_call` to ensure correct timing and execution order.
- Upgraded indent lexer to handle multi-line statements and improve indentation handling.
- Upgraded `parse_map_value` to handle multi-line values and improve parsing logic in Parser.
- Added `log_message_with_trace` function to log messages with informations when running commands with `debug` flag.

### 🧰 Commands

- Added `login` command to authenticate users to install protected or private packages.
- Refactored `install` command to support installing banks, presets and plugins.
  - `install bank <bank_author>.<bank_name>` to install a specific bank of sounds.
  - `install preset <preset_author>.<preset_name>` to install a specific preset.
  - `install plugin <plugin_author>.<plugin_name>` to install a specific plugin.
- Implemented `debug` and `compress` arguments for `build`, `check` and `play` commands.
  - `build --debug` to build the AST with debug information.
  - `check --debug` to check the syntax with debug information.
  - `play --debug` to play the audio with debug information.
  - `build --compress` to compress the output.
  - `check --compress` to compress the output.
  - `play --compress` to compress the output.

## Version 0.0.1-alpha.13 (2025-07-26)

### 🧩 Language Features

- Added support for `fn` directive to define functions in Devalang.
  - Example: `fn myFunction(param1, param2):`

### 🧠 Core Engine

- Patched `trigger`, `call`, and `spawn`, `renderer` to handle correct cursor time in the audio interpreter.
- Refactored audio engine and interpreter to handle correct timing and execution while using `loop`, `call`, and `spawn` statements.
- Refactored `trigger` effects to apply more effects to triggers.
  - Example: `.myTrigger auto { reverb: 0.25, pitch: 0.75, gain: 0.8 }`
- Refactored `preprocessor` to handle correct namespaced banks of sounds and triggers.
- Refactored `collect_errors_recursively` to provide detailed error reporting across nested statements.
- Optimized the `renderer` to handle silent buffers and improve performance.

### 🛠️ Utilities

- Added the `extract_loop_body_statements` utility for better loop handling.
- Improved logging for module variables and functions.

### 🧩 Web Assembly

- Patched `lib.rs` dependencies to ensure compatibility with the latest Rust and WASM standards.

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
