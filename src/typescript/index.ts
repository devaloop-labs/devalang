/**
 * @devaloop/devalang - TypeScript API
 * 
 * Main entry point for the Devalang TypeScript/JavaScript API.
 * Provides access to WASM bindings and high-level wrappers.
 * 
 * @example
 * ```typescript
 * import * as devalang from '@devaloop/devalang';
 * 
 * // Parse Devalang code
 * const result = devalang.parse('test.deva', 'bpm 120\nsleep 1b');
 * 
 * // Render audio
 * const audioBuffer = devalang.renderAudio('bpm 120\nlet s = synth sine {}\ns -> note(C4, { duration: 500 })');
 * ```
 */

export * from './api';
export * from './types';

// Re-export WASM bindings (when built)
export * as wasm from './wasm';
