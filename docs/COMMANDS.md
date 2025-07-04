<div align="center">
    <img src="https://firebasestorage.googleapis.com/v0/b/devaloop-labs.firebasestorage.app/o/devalang-teal-logo.svg?alt=media&token=d2a5705a-1eba-4b49-88e6-895a761fb7f7" alt="Devalang Logo">
</div>

# Devalang Commands Guide

## Initialization

Initialize a new Devalang project (current folder)

```bash
devalang init
```

Initialize a new Devalang project (new folder)

```bash
devalang init --name <project-name> --template <template-name>
```

Available arguments:

- `--name`: The name of the project (cannot be empty)
- `--template`: The template to use for the project (default to `welcome`)

## Checking

Checking syntax of .deva file(s)

```bash
devalang check --entry ./examples --output ./output --watch
```

Available arguments :

- `--no-config`: Whether to ignore the configuration file (default to `false`)
- `--entry`: The input folder (default to `./src`)
- `--output`: The output folder (default to `./output`)
- `--watch`: Whether to watch for changes and re-analyze (default to `false`)

## Building

Building AST of .deva file(s)

```bash
devalang build --entry ./examples --output ./output --watch
```

Available arguments :

- `--no-config`: Whether to ignore the configuration file (default to `false`)
- `--entry`: The input folder (default to `./src`)
- `--output`: The output folder (default to `./output`)
- `--watch`: Whether to watch for changes and rebuild (default to `false`)


## Playing

Playing .deva file(s) without audio playback (once)

```bash
devalang play --entry ./examples --output ./output
```

Playing .deva file(s) with audio playback (once by file change)

```bash
devalang play --entry ./examples --output ./output --watch
```

Playing .deva file(s) with audio playback (infinite loop)

```bash
devalang play --entry ./examples --output ./output --repeat
```

Note : You cannot use `--watch` and `--repeat` options together. Use `--repeat` instead.

Available arguments :

- `--no-config`: Whether to ignore the configuration file (default to `false`)
- `--entry`: The input folder (default to `./src`)
- `--output`: The output folder (default to `./output`)
- `--watch`: Whether to watch for changes and rebuild + play (default to `false`)
- `--repeat`: Whether to repeat the playback of the audio file (default to `false`)