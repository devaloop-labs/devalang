<div align="center">
    <img src="https://firebasestorage.googleapis.com/v0/b/devaloop-labs.firebasestorage.app/o/devalang-teal-logo.svg?alt=media&token=d2a5705a-1eba-4b49-88e6-895a761fb7f7" alt="Devalang Logo">
</div>

# Devalang Syntax Guide

Devalang supports a simple and intuitive syntax for composing music and sound design. Below is a guide to the basic syntax elements, types, and usage examples.

The engine is designed to be easy to read and write, allowing you to focus on your music rather than the code.

The engine uses indentation to define blocks, similar to Python. Each block must be indented consistently.

➡️ For full examples, check the `examples/` folder of the repository.

## Types

<details>
<summary>Show available types</summary>

### String

Strings are defined using double quotes.

```deva
let string = "myValue"
```

### Number

Numbers can be integers or floating-point values. They do not require quotes.

```deva
let number = 99
```

### Boolean

Booleans can be either `true` or `false` without quotes.

```deva
let boolean = false
```

### Map

Maps are key-value pairs defined using curly braces. Keys are strings, and values can be of any type (string, number, boolean, map, or array).

```deva
let map = {myKey: 99}
```

### Array

Arrays are ordered lists of values defined using square brackets. Values can be of any type (string, number, boolean, map, or array).

```deva
let array = [3, 4]
```

</details>

## Syntax usage

### Beats Per Minute

BPM is used to set the global tempo of the music.

```deva
bpm 125
```

### Sound Bank

Bank is used to select a sound bank for the audio engine.

```deva
bank 808
```

### Importing / Exporting Modules

Modules can be imported and exported to share variables between different files.

Exporting variables from a module :

```deva
# exported.deva

let exportedIterator = 10
let exportedParams = {drive: 50, decay: 30}

@export { exportedIterator, exportedParams }
```

Importing and using the exported variables in another module :

```deva
# index.deva

@import { exportedIterator, exportedParams } "./exported.deva"

loop exportedIterator:
    .kick auto exportedParams
```

### Built-in triggers

Usage : `.<trigger-name> <duration> <effects-map>`

Other triggers will be added in future releases (e.g. `.snare`, `.hihat`, `.tom`, `.clap`, `.crash`, `.ride`, `.synth`, `.bass`, `.pad`).
You can also create custom triggers using the `@load` directive.

```deva
.kick
.kick 1/4
.kick auto {reverb: 50, decay: 30}
```

### Custom triggers

Same usage as built-in triggers, but with custom audio files or effects.

```deva
@load "./path/to/instrument.mp3" as mySample

.mySample auto {reverb: 50, drive: 25}
```

### Let variables

Variables are defined using the `let` keyword, followed by the variable name and its value. The value can be of any type (string, number, boolean, map, or array).

```deva
let number = 0
let boolean = true
let string = "string"
let map = {myKey: 200}
let array = [0, 1, 2]
```

### Basic loops

Loops are defined using the `loop` keyword, followed by the number of iterations. The body of the loop is indented.

```deva
loop 10:
    # ...
```
