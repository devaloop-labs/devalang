#!/usr/bin/env node
"use strict";
/**
 * Devalang CLI entry point
 *
 * This is a placeholder that forwards to the Rust CLI binary.
 * The actual CLI is built with Cargo and should be run with:
 *
 *   cargo run --features cli -- [args]
 *
 * For the npm package, users should install the Rust toolchain
 * or we should provide pre-built binaries.
 */
Object.defineProperty(exports, "__esModule", { value: true });
console.log('Devalang CLI');
console.log('============\n');
console.log('The Devalang CLI is built with Rust.');
console.log('Please run it using Cargo:\n');
console.log('  cargo run --features cli -- build --path examples/test.deva\n');
console.log('Or install the standalone binary from:');
console.log('  https://github.com/devaloop-labs/devalang/releases\n');
process.exit(0);
//# sourceMappingURL=index.js.map