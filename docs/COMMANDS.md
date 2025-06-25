<div align="center">
    <img src="https://firebasestorage.googleapis.com/v0/b/devaloop-labs.firebasestorage.app/o/devalang-teal-logo.svg?alt=media&token=d2a5705a-1eba-4b49-88e6-895a761fb7f7" alt="Devalang Logo">
</div>

# Devalang Commands Guide

## Checking

Checking syntax of .deva file(s)

```bash
devalang check --entry ./examples --output ./output
```

Available arguments :

- `entry`: The input folder (default to `./src`)
- `output`: The output folder (default to `./output`)

## Building

Building AST of .deva file(s)

```bash
devalang build --entry ./examples --output ./output
```

Available arguments :

- `entry`: The input folder (default to `./src`)
- `output`: The output folder (default to `./output`)
