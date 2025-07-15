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

### Duration

Duration is defined using a number. It represents the length of a sound in milliseconds.

```deva
let duration = 1000
```

Will play a sound for 1000 milliseconds (1 second).

NOTE: Other time units like seconds or beats are not supported yet, but will be in the future.

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
let map = { myKey: 99 }
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

> ⚠️ The import/export system is still experimental and may change in the future.
>
> You must explicitly declare imports and exports in each file — Devalang does not automatically detect or resolve them.

Exporting variables from a module :

```deva
# exported.deva

let exportedIterator = 10
let exportedParams = { drive: 50, decay: 30 }

@export { exportedIterator, exportedParams }
```

Importing and using the exported variables in another module :

```deva
# index.deva

@import { exportedIterator, exportedParams } "./exported.deva"

loop exportedIterator:
    .kick auto exportedParams
```

### Loading Samples

You can load your own samples and use them in your music.

Load usage : `@load <path> as <name>`

Trigger usage : `.<name> <duration> <params>`

```deva
@load "./path/to/instrument.mp3" as mySample

.mySample auto {reverb: 50, drive: 25}
```

### Variables

Variables are defined using the `let` keyword, followed by the variable name and its value. The value can be of any type (string, number, boolean, map, or array).

```deva
let number = 0
let boolean = true
let string = "string"
let map = { myKey: 200 }
let array = [0, 1, 2]
```

### Loops

Loops are defined using the `loop` keyword, followed by the number of iterations. The body of the loop is indented.

```deva
loop 10:
    # ...
```

### Groups

Groups are defined using the `group` keyword, followed by the group name. The body of the group is indented.

Groups allow you to organize your code into reusable blocks. They can be called or spawned later in the code.

```deva
group myGroup:
    # ...
```

### Conditions

Conditions are defined using the `if` keyword, followed by a condition. The body of the condition is indented.

```deva
if myCondition:
    # ...
```

You can also use `else` and `else if` for alternative branches:

```deva
if myCondition:
    # ...
else if anotherCondition:
    # ...
else:
    # ...
```

### Calling Groups (Sequential Execution)

Groups can be called using the `call` keyword, which executes only the group in sequence.

> ⚠️ `call` only works on `group` declarations. It does not apply to other statements.

This executes the entire group in the current execution thread, following a sequential order.

```deva
call myGroup
```

### Spawning Groups (Parallel Execution)

Groups can be spawned using the `spawn` keyword, which executes only the group in parallel.

> ⚠️ spawn also only works on group declarations. It does not make the group’s content parallel unless it explicitly uses spawn inside.

This runs the entire group in a separate execution thread, allowing it to play alongside other actions.

```deva
spawn myGroup
```

### Synthesizers

Synthesizers are defined using the `synth` keyword, followed by the synthesizer type. You can create a synthesizer instance and use it to play notes.

```deva
let mySynth = synth sine

mySynth -> note(C4, { duration: 400 })
mySynth -> note(G4, { duration: 400 })
mySynth -> note(E4, { duration: 600 })
mySynth -> note(A4, { duration: 400 })
mySynth -> note(F4, { duration: 800 })
mySynth -> note(D4, { duration: 400 })
mySynth -> note(B3, { duration: 600 })

# You can use call or spawn to execute the synthesizer actions in sequence or parallel.

call mySynth
```
