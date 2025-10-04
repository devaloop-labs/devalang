/**
 * WASM bindings re-export
 * 
 * This module re-exports the generated WASM bindings.
 * It will be available after running: npm run rust:wasm:all
 */

// These will be available after WASM build
export * from '../../pkg/node/devalang_wasm';
