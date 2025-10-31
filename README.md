<div align="center">
    <img src="https://devalang.com/images/devalang-logo-min.png" alt="Devalang Logo" width="100" />
</div>

![Rust](https://img.shields.io/badge/Made%20with-Rust-%23DEA584?logo=rust)
![TypeScript](https://img.shields.io/badge/Built%20with-TypeScript-%233178C6?logo=typescript&logoColor=white)

![Rust](https://img.shields.io/badge/Rust-1.70.0%2B-%23DEA584?logo=rust)
![Node.js](https://img.shields.io/badge/Node.js-18%2B-%2368A063?logo=node.js&logoColor=white)

![Project Status](https://img.shields.io/badge/status-preview-%237C5FFF)
![Version](https://img.shields.io/npm/v/@devaloop/devalang?label=version&color=%237C5FFF)
![License: MIT](https://img.shields.io/badge/license-MIT-%237C5FFF)

![Platforms](https://img.shields.io/badge/platform-Windows%20%2F%20Linux%20%2F%20macOS-blue?logo=data:image/svg+xml;base64,PHN2ZyB4bWxucz0iaHR0cDovL3d3dy53My5vcmcvMjAwMC9zdmciIGhlaWdodD0iMjRweCIgdmlld0JveD0iMCAtOTYwIDk2MCA5NjAiIHdpZHRoPSIyNHB4IiBmaWxsPSIjRkZGRkZGIj48cGF0aCBkPSJNMjQwLTEyMHYtODBsNDAtNDBIMTYwcS0zMyAwLTU2LjUtMjMuNVQ4MC0zMjB2LTQ0MHEwLTMzIDIzLjUtNTYuNVQxNjAtODQwaDY0MHEzMyAwIDU2LjUgMjMuNVQ4ODAtNzYwdjQ0MHEwIDMzLTIzLjUgNTYuNVQ4MDAtMjQwSDY4MGw0MCA0MHY4MEgyNDBabS04MC0yMDBoNjQwdi00NDBIMTYwdjQ0MFptMCAwdi00NDAgNDQwWiIvPjwvc3ZnPg==)

![GitHub Actions Workflow Status](https://img.shields.io/github/actions/workflow/status/devaloop-labs/devalang/.github/workflows/ci.yml?label=CI&logo=github)
![Coverage](https://img.shields.io/badge/coverage-TBD-red?label=coverage&logo=codecov&logoColor=white)

![npm](https://img.shields.io/npm/dt/@devaloop/devalang?label=npm&color=%231DB954)
![crates](https://img.shields.io/crates/d/devalang?label=crates.io&color=%231DB954)

# 🦊 Devalang — Write music with code

Devalang is a compact **domain-specific language** (DSL) for **music makers**, **sound designers**, and **creative coders**.
Compose loops, control samples, synthesize audio, and render your ideas — all in clean, **readable text**.

Whether you're **prototyping a beat**, building **generative music**, or **performing live**, Devalang gives you rhythmic precision with the elegance of code.

**From studio sketches to live sets, Devalang puts musical ideas into motion.**

## 📚 Quick Access

### Websites & Resources

[![Website](https://img.shields.io/badge/Official%20Website-%233D2E81?style=for-the-badge&logo=data:image/svg+xml;base64,PHN2ZyB4bWxucz0iaHR0cDovL3d3dy53My5vcmcvMjAwMC9zdmciIGhlaWdodD0iMjBweCIgdmlld0JveD0iMCAtOTYwIDk2MCA5NjAiIHdpZHRoPSIyMHB4IiBmaWxsPSIjRkZGRkZGIj48cGF0aCBkPSJNNDgwLjI4LTk2UTQwMS05NiAzMzEtMTI2dC0xMjIuNS04Mi41UTE1Ni0yNjEgMTI2LTMzMC45NnQtMzAtMTQ5LjVROTYtNTYwIDEyNi02MjkuNXEzMC02OS41IDgyLjUtMTIyVDMzMC45Ni04MzRxNjkuOTYtMzAgMTQ5LjUtMzB0MTQ5LjA0IDMwcTY5LjUgMzAgMTIyIDgyLjVUODM0LTYyOS4yOHEzMCA2OS43MyAzMCAxNDlRODY0LTQwMSA4MzQtMzMxdC04Mi41IDEyMi41UTY5OS0xNTYgNjI5LjI4LTEyNnEtNjkuNzMgMzAtMTQ5IDMwWm0tLjI4LTcycTEyMiAwIDIxMC04MXQxMDAtMjAwcS05IDgtMjAuNSAxMi41VDc0NC00MzJINjAwcS0yOS43IDAtNTAuODUtMjEuMTVRNTI4LTQ3NC4zIDUyOC01MDR2LTQ4SDM2MHYtOTZxMC0yOS43IDIxLjE1LTUwLjg1UTQwMi4zLTcyMCA0MzItNzIwaDQ4di0yNHEwLTE0IDUtMjZ0MTMtMjFxLTMtMS0xMC0xaC04cS0xMzAgMC0yMjEgOTF0LTkxIDIyMWgyMTZxNjAgMCAxMDIgNDJ0NDIgMTAydjQ4SDM4NHYxMDVxMjMgOCA0Ni43MyAxMS41UTQ1NC40NS0xNjggNDgwLTE2OFoiLz48L3N2Zz4=&logoColor=white)](https://devalang.com)
[![Docs](https://img.shields.io/badge/Documentation-%235A44BA?style=for-the-badge&logo=data:image/svg+xml;base64,PHN2ZyB4bWxucz0iaHR0cDovL3d3dy53My5vcmcvMjAwMC9zdmciIGhlaWdodD0iMjBweCIgdmlld0JveD0iMCAtOTYwIDk2MCA5NjAiIHdpZHRoPSIyMHB4IiBmaWxsPSIjRkZGRkZGIj48cGF0aCBkPSJNNDQ0LTI0NnYtNDU0cS00Mi0yMi04Ny0zM3QtOTMuMjItMTFxLTM2Ljk0IDAtNzMuMzYgNi41VDEyMC03MTZ2NDUycTM1LTEzIDcwLjgxLTE4LjVRMjI2LjYzLTI4OCAyNjQtMjg4cTQ3LjM1IDAgOTIuMTcgMTJRNDAxLTI2NCA0NDQtMjQ2Wm0zNiAxMDJxLTQ5LTMyLTEwMy01MnQtMTEzLTIwcS0zOCAwLTc2IDcuNVQxMTUtMTg2cS0yNCAxMC00NS41LTMuNTNUNDgtMjI5di01MDNxMC0xNCA3LjUtMjZUNzYtNzc2cTQ1LTIwIDkxLjktMzAgNDYuOTEtMTAgOTUuNjgtMTBRMzMzLTgxNiAzODQtODAyLjVUNDkyLTc2MHExMSA2IDE3LjUgMTYuNVQ1MTYtNzIwdjQ3NHE0My0yMCA4Ny44My0zMSA0NC44Mi0xMSA5Mi4xNy0xMSAzNyAwIDczLjUgNXQ3MC41IDE5di01MjlxMTEgNCAyMi4xMyA3LjkgMTEuMTMgMy45IDIxLjg3IDkuMSAxMyA2IDIxIDE4dDggMjZ2NTAzcTAgMjUtMTUuNSA0MHQtMzIuNSA3cS00MC0xOC04Mi40OC0yNi00Mi40Ny04LTg2LjUyLTgtNTkgMC0xMTMgMjB0LTEwMyA1MlptMTQ0LTI0MHYtNDMybDEyMC00OHY0MzJsLTEyMCA0OFpNMjgyLTQ5NVoiLz48L3N2Zz4=&logoColor=white)](https://docs.devalang.com)
[![Playground](https://img.shields.io/badge/Playground-%237C5FFF?style=for-the-badge&logo=data:image/svg+xml;base64,PHN2ZyB4bWxucz0iaHR0cDovL3d3dy53My5vcmcvMjAwMC9zdmciIGhlaWdodD0iMjRweCIgdmlld0JveD0iMCAtOTYwIDk2MCA5NjAiIHdpZHRoPSIyNHB4IiBmaWxsPSIjRkZGRkZGIj48cGF0aCBkPSJtMjcyLTQ0MCAyMDggMTIwIDIwOC0xMjAtMTY4LTk3djEzN2gtODB2LTEzN2wtMTY4IDk3Wm0xNjgtMTg5di0xN3EtNDQtMTMtNzItNDkuNVQzNDAtNzgwcTAtNTggNDEtOTl0OTktNDFxNTggMCA5OSA0MXQ0MSA5OXEwIDQ4LTI4IDg0LjVUNTIwLTY0NnYxN2wyODAgMTYxcTE5IDExIDI5LjUgMjkuNVQ4NDAtMzk4djc2cTAgMjItMTAuNSA0MC41VDgwMC0yNTJMNTIwLTkxcS0xOSAxMS00MCAxMXQtNDAtMTFMMTYwLTI1MnEtMTktMTEtMjkuNS0yOS41VDEyMC0zMjJ2LTc2cTAtMjIgMTAuNS00MC41VDE2MC00NjhsMjgwLTE2MVptMCAzNzhMMjAwLTM4OXY2N2wyODAgMTYyIDI4MC0xNjJ2LTY3TDUyMC0yNTFxLTE5IDExLTQwIDExdC00MC0xMVptNDAtNDY5cTI1IDAgNDIuNS0xNy41VDU0MC03ODBxMC0yNS0xNy41LTQyLjVUNDgwLTg0MHEtMjUgMC00Mi41IDE3LjVUNDIwLTc4MHEwIDI1IDE3LjUgNDIuNVQ0ODAtNzIwWm0wIDU2MFoiLz48L3N2Zz4=&logoColor=white)](https://playground.devalang.com)

### Important files

[![Changelog](https://img.shields.io/badge/Changelog-%23129978?style=for-the-badge&logo=data:image/svg+xml;base64,PHN2ZyB4bWxucz0iaHR0cDovL3d3dy53My5vcmcvMjAwMC9zdmciIGhlaWdodD0iMjRweCIgdmlld0JveD0iMCAtOTYwIDk2MCA5NjAiIHdpZHRoPSIyNHB4IiBmaWxsPSIjRkZGRkZGIj48cGF0aCBkPSJNMjQwLTgwcS01MCAwLTg1LTM1dC0zNS04NXYtMTIwaDEyMHYtNTYwbDYwIDYwIDYwLTYwIDYwIDYwIDYwLTYwIDYwIDYwIDYwLTYwIDYwIDYwIDYwLTYwIDYwIDYwIDYwLTYwdjY4MHEwIDUwLTM1IDg1dC04NSAzNUgyNDBabTQ4MC04MHExNyAwIDI4LjUtMTEuNVQ3NjAtMjAwdi01NjBIMzIwdjQ0MGgzNjB2MTIwcTAgMTcgMTEuNSAyOC41VDcyMC0xNjBaTTM2MC02MDB2LTgwaDI0MHY4MEgzNjBabTAgMTIwdi04MGgyNDB2ODBIMzYwWm0zMjAtMTIwcS0xNyAwLTI4LjUtMTEuNVQ2NDAtNjQwcTAtMTcgMTEuNS0yOC41VDY4MC02ODBxMTcgMCAyOC41IDExLjVUNzIwLTY0MHEwIDE3LTExLjUgMjguNVQ2ODAtNjAwWm0wIDEyMHEtMTcgMC0yOC41LTExLjVUNjQwLTUyMHEwLTE3IDExLjUtMjguNVQ2ODAtNTYwcTE3IDAgMjguNSAxMS41VDcyMC01MjBxMCAxNy0xMS41IDI4LjVUNjgwLTQ4MFpNMjQwLTE2MGgzNjB2LTgwSDIwMHY0MHEwIDE3IDExLjUgMjguNVQyNDAtMTYwWm0tNDAgMHYtODAgODBaIi8+PC9zdmc+)](https://github.com/devaloop-labs/devalang/blob/main/docs/CHANGELOG.md)
[![Examples](https://img.shields.io/badge/Examples-%231DE9B6?style=for-the-badge&logo=data:image/svg+xml;base64,PHN2ZyB4bWxucz0iaHR0cDovL3d3dy53My5vcmcvMjAwMC9zdmciIGhlaWdodD0iMjRweCIgdmlld0JveD0iMCAtOTYwIDk2MCA5NjAiIHdpZHRoPSIyNHB4IiBmaWxsPSIjRkZGRkZGIj48cGF0aCBkPSJNNDgwLTgwcS0zMyAwLTU2LjUtMjMuNVQ0MDAtMTYwaDE2MHEwIDMzLTIzLjUgNTYuNVQ0ODAtODBaTTMyMC0yMDB2LTgwaDMyMHY4MEgzMjBabTEwLTEyMHEtNjktNDEtMTA5LjUtMTEwVDE4MC01ODBxMC0xMjUgODcuNS0yMTIuNVQ0ODAtODgwcTEyNSAwIDIxMi41IDg3LjVUNzgwLTU4MHEwIDgxLTQwLjUgMTUwVDYzMC0zMjBIMzMwWm0yNC04MGgyNTJxNDUtMzIgNjkuNS03OVQ3MDAtNTgwcTAtOTItNjQtMTU2dC0xNTYtNjRxLTkyIDAtMTU2IDY0dC02NCAxNTZxMCA1NCAyNC41IDEwMXQ2OS41IDc5Wm0xMjYgMFoiLz48L3N2Zz4=)](https://github.com/devaloop-labs/devalang/blob/main/examples)

### Common projects and tools

[![Devapack](https://img.shields.io/badge/Devapack-%235A44BA?style=for-the-badge&logo=data:image/svg+xml;base64,PHN2ZyB4bWxucz0iaHR0cDovL3d3dy53My5vcmcvMjAwMC9zdmciIGhlaWdodD0iMjRweCIgdmlld0JveD0iMCAtOTYwIDk2MCA5NjAiIHdpZHRoPSIyNHB4IiBmaWxsPSIjRkZGRkZGIj48cGF0aCBkPSJNNDQwLTE4M3YtMjc0TDIwMC01OTZ2Mjc0bDI0MCAxMzlabTgwIDAgMjQwLTEzOXYtMjc0TDUyMC00NTd2Mjc0Wm0tODAgOTJMMTYwLTI1MnEtMTktMTEtMjkuNS0yOVQxMjAtMzIxdi0zMThxMC0yMiAxMC41LTQwdDI5LjUtMjlsMjgwLTE2MXExOS0xMSA0MC0xMXQ0MCAxMWwyODAgMTYxcTE5IDExIDI5LjUgMjl0MTAuNSA0MHYzMThxMCAyMi0xMC41IDQwVDgwMC0yNTJMNTIwLTkxcS0xOSAxMS00MCAxMXQtNDAtMTFabTIwMC01MjggNzctNDQtMjM3LTEzNy03OCA0NSAyMzggMTM2Wm0tMTYwIDkzIDc4LTQ1LTIzNy0xMzctNzggNDUgMjM3IDEzN1oiLz48L3N2Zz4=)](https://github.com/devaloop-labs/devapack)
[![VSCode Extension](https://img.shields.io/badge/VSCode%20Extension-%237C5FFF?style=for-the-badge&logo=data:image/svg+xml;base64,PHN2ZyB4bWxucz0iaHR0cDovL3d3dy53My5vcmcvMjAwMC9zdmciIGhlaWdodD0iMjRweCIgdmlld0JveD0iMCAtOTYwIDk2MCA5NjAiIHdpZHRoPSIyNHB4IiBmaWxsPSIjRkZGRkZGIj48cGF0aCBkPSJNMzUyLTEyMEgyMDBxLTMzIDAtNTYuNS0yMy41VDEyMC0yMDB2LTE1MnE0OCAwIDg0LTMwLjV0MzYtNzcuNXEwLTQ3LTM2LTc3LjVUMTIwLTU2OHYtMTUycTAtMzMgMjMuNS01Ni41VDIwMC04MDBoMTYwcTAtNDIgMjktNzF0NzEtMjlxNDIgMCA3MSAyOXQyOSA3MWgxNjBxMzMgMCA1Ni41IDIzLjVUODAwLTcyMHYxNjBxNDIgMCA3MSAyOXQyOSA3MXEwIDQyLTI5IDcxdC03MSAyOXYxNjBxMCAzMy0yMy41IDU2LjVUNzIwLTEyMEg1NjhxMC01MC0zMS41LTg1VDQ2MC0yNDBxLTQ1IDAtNzYuNSAzNVQzNTItMTIwWm0tMTUyLTgwaDg1cTI0LTY2IDc3LTkzdDk4LTI3cTQ1IDAgOTggMjd0NzcgOTNoODV2LTI0MGg4MHE4IDAgMTQtNnQ2LTE0cTAtOC02LTE0dC0xNC02aC04MHYtMjQwSDQ4MHYtODBxMC04LTYtMTR0LTE0LTZxLTggMC0xNCA2dC02IDE0djgwSDIwMHY4OHE1NCAyMCA4NyA2N3QzMyAxMDVxMCA1Ny0zMyAxMDR0LTg3IDY4djg4Wm0yNjAtMjYwWiIvPjwvc3ZnPg==)](https://marketplace.visualstudio.com/items?itemName=devaloop.devalang-vscode)

### Downloads

[![Binaries & Installers](https://img.shields.io/badge/Binaries%20%26%20installers-%23129978?style=for-the-badge&logo=data:image/svg+xml;base64,PHN2ZyB4bWxucz0iaHR0cDovL3d3dy53My5vcmcvMjAwMC9zdmciIGhlaWdodD0iMjRweCIgdmlld0JveD0iMCAtOTYwIDk2MCA5NjAiIHdpZHRoPSIyNHB4IiBmaWxsPSIjRkZGRkZGIj48cGF0aCBkPSJNNDgwLTMyMCAyODAtNTIwbDU2LTU4IDEwNCAxMDR2LTMyNmg4MHYzMjZsMTA0LTEwNCA1NiA1OC0yMDAgMjAwWk0yNDAtMTYwcS0zMyAwLTU2LjUtMjMuNVQxNjAtMjQwdi0xMjBoODB2MTIwaDQ4MHYtMTIwaDgwdjEyMHEwIDMzLTIzLjUgNTYuNVQ3MjAtMTYwSDI0MFoiLz48L3N2Zz4=)](https://devalang.com/download)

### Community

[![Discord](https://img.shields.io/badge/Labscend-%235865F2?style=for-the-badge&logo=discord&logoColor=white)](https://discord.gg/wCqd8e6JD8)
[![Instagram](https://img.shields.io/badge/Labscend-%23E1306C?style=for-the-badge&logo=Instagram&logoColor=white)](https://www.instagram.com/labscend)
[![X](https://img.shields.io/badge/Labscend-%23000?style=for-the-badge&logo=X&logoColor=white)](https://x.com/labscend)
[![LinkedIn](https://img.shields.io/badge/Labscend%20studios-%230077B5?style=for-the-badge&logo=data:image/png;base64,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)](https://linkedin.com/company/labscend-studios)


## ⚡ Quick Start

### [Try in Your Browser](https://playground.devalang.com)

Launch the Devalang Playground to write, run, and share Devalang code directly in your web browser. No installation required !

### [Download an installer (recommended)](https://devalang.com/download)

Visit the download page to get the latest releases for Windows, macOS, and Linux.

This is the recommended way to install Devalang, as the installers will set up everything for you, including adding Devalang to your system PATH.

### [Download a binary executable (advanced)](https://devalang.com/download)

You can also download the standalone binaries for your OS from the download page.

You must ensure that the binary is executable (e.g., `chmod +x <binary_name>` on Linux/Mac).

You must also add the binary to your system PATH to use it from any terminal location.

---

### Install via npm (Node.js)

```bash
npm install -g @devaloop/devalang
```

### Install via Cargo (Rust)

```bash
cargo install devalang
```

### (optional but recommended) Install the VSCode extension

```bash
code --install-extension devaloop.devalang-vscode
```

### Create Your First Project

```bash
# Initialize a new project
devalang init my-project

# Navigate to the project
cd my-project

# Check syntax
devalang check --entry examples/index.deva

# Build audio files
devalang build --path examples/index.deva --formats wav mid

# Play audio (live mode)
devalang play --live --input examples/index.deva
```

## 📦 (optional) Install addons

Devalang supports addons to extend functionalities. This allows you to easily add sound banks, effects, or other features.

> To create your own addon, please refer to the [Devapack documentation](https://github.com/devaloop-labs/devapack/tree/main/docs).

```bash
# List available addons
devalang addon list

# Install an addon (format: <author>.<addon-name>)
devalang addon install devaloop.808
```

This will install the `devaloop.808` sound bank in your current working directory inside `.deva` folder.

**You can then use it in your Devalang scripts !**

## 🎵 Your First Devalang File

Create a file `hello.deva` or `index.deva` (if you do not specify `--input` argument, it defaults to `index.deva`).

#### Nomenclature for .deva files

- Devalang files use the `.deva` extension.
- Devalang engine is **indentation-sensitive** for blocks, similar to Python.
- Files are plain text and can be edited with **any text editor** (VSCode recommended).
- Ensure your text editor supports **UTF-8 encoding**.
- Devalang is **case-sensitive**, so be consistent with capitalization.
- Devalang reads files from **top to bottom**, so order matters.
- Devalang files typically starts with :
  1. Module system (`@load`, `@import`, `@use`)
  2. Global settings (`bpm`, `bank`)
  3. Definitions (`const/let/var`, `synth`, `pattern`, `group`)
  4. Algorithmic logic (`if`, `loop`, `for`)
  5. Musical logic / Execution (trigger calls, automations, `call`, `spawn`, `sleep`)
  6. Optional exports (`@export`)
- Devalang files can include comments using `#` or `//` for single-line comments.
- You can name your files anything, but `index.deva` is a common convention for the main entry file.
- You can organize your project with subfolders as needed. (use module system like `@import { var } from '<module_file_path>.deva'` and `@export { var }`).

Refer to the [documentation](https://docs.devalang.com) for a complete syntax reference.

```deva
# Import some variables from other modules
@import { myTempo } from "./shared/variables.deva" 

# Load an external sample and a MIDI file
@load "./samples/my-sample.wav" as mySample
@load "./midi/my-midi-file.mid" as myMidiFile

# Set the tempo with the imported variable
bpm myTempo

# Load a bank of sounds (make sure you have the bank installed)
bank devaloop.808 as drums

# Create a simple kick pattern (can also use "mySample")
pattern kickPattern with drums.kick = "x--- x--- x--- x---"

# Define a constant for synth, with a global LFO effect
# You can define variables scopes using const/let/var
const mySynth = synth saw
  -> lfo({
    rate: "1/8",
    depth: 1.0,
    waveform: "triangle",
    target: "pitch",
    semitones: 3.0
  })

# Define a melody using a group to organize notes
group myMelody:
  mySynth -> note(C5)
          -> duration(500)            # Note playing for 500ms

  mySynth -> note(E5)
          -> duration(1/4)            # Note playing for 1/4 beats

  mySynth -> chord(Gmaj7)
          -> duration(1/8)            # Chord playing for 1/8 beats
          -> velocity(100)            # Velocity (0.0 to 1.0) or 0-127
          -> lpf({
              cutoff: 800.0,          # Lowpass filter at 800 Hz
              resonance: 0.5          # Filter resonance at 50%
            })
          -> reverb({ size: 0.3 })    # Small reverb effect

# Play the melody (in parallel)
spawn myMelody

# Play the kick pattern (in parallel too)
spawn kickPattern

# Bind and play the loaded MIDI pattern with 'mySynth' synth
bind myMidiFile -> mySynth

# Pause for 1/4 beats
sleep 1/4

# Store the sample in a variable and apply effects to it
let myAwesomeSample = .mySample
    -> reverse(true)                  # Reverse the sample audio
    -> speed(2.0)                     # Multiply the playing speed by 2
    -> dist({
        amount: 1.0,                  # Apply a maximal distortion
        mix: 0.5                      # Apply a 50% mixing
    })
    -> reverb({
        size: 1.0,                    # Apply a maximal reverb size
        decay: 0.1,                   # Apply a short decay
        mix: 0.5                      # Apply a 50% mixing
    })
    -> reverb({                       # Duplicate reverb for a stronger effect
        size: 1.0,
        decay: 0.1,
        mix: 0.5
    })

# Playing the stored sample trigger in different ways
.myAwesomeSample                        # Play the full sample length
.myAwesomeSample auto                   # Use maximal sample length by default
.myAwesomeSample 1/8                    # Play the sample for 1/8 beats

# Play the sample in conditional loop
for i in [0..3]:
    if i == 2:
        .myAwesomeSample 1/4            # Play for 1/4 beats on iteration 2
    else:
        .myAwesomeSample 1/8            # Play for 1/8 beats otherwise

# Play the sample in a blocking loop (run 10 times before continuing)
loop 10:
    .myAwesomeSample auto

# Play the sample in an (infinite) passthrough loop (non-blocking)
# This will continue playing in the background and let the script continue
# You can also specify a passthrough max duration using "loop pass(500):" (in ms)
loop pass:
    .myAwesomeSample auto

# Export the melody
@export { myMelody }
```

### ⚙️ (optional) Configure project settings

You can create a `devalang.json` (recommended) or `devalang.toml` or even `.devalang` (legacy) file to customize check/build/play settings.

This typically evitate to re-type common arguments like `--path`, `--formats`, etc.

> Comments are not supported in config files, please use `devalang init` to generate a default config.

```jsonc
{
  "project": {
    "name": "My Awesome Project"        // Change this to adjust project name
  },
  "paths": {
    "entry": "audio/helloWorld.deva",   // Change this to adjust entry file path
    "output": "output"                  // Change this to adjust output directory
  },
  "audio": {
    "format": ["wav", "mid"],           // Change this to adjust output formats (options: wav, mid, mp3)
    "bit_depth": 16,                    // Change this to 24 or 32 for higher quality
    "channels": 2,                      // Change this to 1 for mono output
    "sample_rate": 44100,               // Change this to 48000 for higher quality
    "resample_quality": "sinc24",       // Change this to adjust resampling quality (options: sinc8, sinc16, sinc24, sinc32)
    "bpm": 120                           // Change this to adjust the project tempo (only if not set in code)
  },
  "live": {
    "crossfade_ms": 50                  // Change this to adjust crossfade duration when playing live
  }
}

```

### Build the audio

```bash
# Build to WAV, MP3, and MIDI
devalang build --path hello.deva --formats wav,mp3,mid
```

### Play the audio

```bash
# Play the audio file
devalang play --input hello.deva

# Play live (repeats and watch until stopped)
devalang play --live --input hello.deva

# Play live loop without crossfade
# With 0ms, transitions between loops are no more distinguishable
devalang play --live --crossfade-ms 0 --input hello.deva
```

## 🚀 Features

### 🎵 **Core Language**
- ✅ **Lexer & Parser** — Complete tokenization and AST generation
- ✅ **Patterns** — Rhythmic notation with swing, humanize, velocity
- ✅ **Synths** — Built-in synthesizers with ADSR envelopes
- ✅ **Filters** — Lowpass, highpass, bandpass audio filtering
- ✅ **Effects** — Reverb, delay, distortion, drive, chorus
- ✅ **Variables** — `let`, `const`, `var` with scoping
- ✅ **Groups & Spawn** — Organize and parallelize execution
- ✅ **Loops & Conditions** — `for`, `if`, `else` control flow
- ✅ **Triggers** — Conditional audio triggering
- ✅ **Events** — Event system with `on` and `emit`

### 🛠️ **CLI Tools**
- ✅ `devalang init` — Scaffold new projects
- ✅ `devalang build` — Compile to WAV/MIDI/MP3
- ✅ `devalang check` — Validate syntax
- ✅ `devalang play` — Audio playback
- ✅ `devalang addon` — Manage addons (install, list, discover)
- ✅ `devalang login/logout` — Authentication
- ✅ `devalang telemetry` — Privacy controls

### 🌐 **WASM API**
- ✅ `render_audio()` — Browser audio rendering
- ✅ `render_midi_array()` — MIDI export
- ✅ `debug_render()` — Debug information
- ✅ `parse()` — Parse Devalang code
- ✅ TypeScript types included

### 📦 **Output Formats**
- ✅ **WAV** — 16/24/32-bit audio export
- ✅ **MIDI** — Standard MIDI file export
- ✅ **MP3** — Lossy audio export (via LAME)

### 🎯 **Performance**
- ⚡ **Fast builds** — 7-10ms for typical projects
- ⚡ **Low latency** — Optimized audio engine
- ⚡ **Release builds** — 5-6x faster than debug

### 📚 **Learning Resources**
- ✅ **Online Docs** — Complete language reference
- ✅ **VSCode Extension** — Syntax highlighting

## 💡 Why Devalang?

- 🎹 **Prototype audio ideas** without opening a DAW
- 💻 **Integrate sound** into code-based workflows
- 🎛️ **Control audio parameters** with readable syntax
- 🧪 **Build musical logic** with variables and conditions
- 🔄 **Create patterns** with expressive notation
- 🎨 **Live code** with fast iteration cycles
- 📦 **Version control** your music with git

## 🔧 Development

### Build from Source

```bash
# Clone the repository
git clone https://github.com/devaloop-labs/devalang.git
cd devalang

# NPM (TypeScript) and Cargo (Rust) are required
npm install

# Build CLI (Rust)
cargo build

# Build WASM (Web & Node.js)
npm run rust:wasm:all

# Build TypeScript
npm run ts:build

# Run tests
cargo test --features cli
npm test
```

## 🤝 Contributing

We welcome contributions! See [CONTRIBUTING.md](./CONTRIBUTING.md) for guidelines.

### Ways to Contribute

- 🐛 **Report bugs** via [GitHub Issues](https://github.com/devaloop-labs/devalang/issues)
- 💡 **Suggest features** on [hello@labscend.studio](mailto://hello@labscend.studio)
- 🎵 **Share examples** of your creations with **#devalang** tag

## 📜 License

MIT License — See [LICENSE](./LICENSE) for details.

Copyright (c) 2025 Labscend Studios

---

<div align="center">
    <strong>Made with ❤️ by <a href="https://labscend.studio">Labscend Studios</a></strong>
    <br />
    <sub>Star ⭐ the repo if you like it !</sub>
</div>
