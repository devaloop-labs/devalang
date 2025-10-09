/* tslint:disable */
/* eslint-disable */
/**
 * Enable hot reload mode with callback
 */
export function enable_hot_reload(callback: Function): void;
/**
 * Disable hot reload mode
 */
export function disable_hot_reload(): void;
/**
 * Check if hot reload is enabled
 */
export function is_hot_reload_enabled(): boolean;
/**
 * Register a JavaScript callback for playhead events
 *
 * The callback will be called for each note on/off event during audio rendering.
 * Call the returned unregister function to remove the callback.
 */
export function register_playhead_callback(callback: Function): any;
/**
 * Collect all playhead events that have been generated
 *
 * Returns an array of playhead events with timing and note information.
 * Events are not cleared after collection - use this for pre-scheduling.
 */
export function collect_playhead_events(): any;
/**
 * Register an audio bank addon
 *
 * Format: "/addons/banks/<publisher>/<name>" or "devalang://bank/<publisher>.<name>"
 * This is a simplified implementation that registers a bank without loading samples.
 * Samples should be registered separately using register_sample().
 */
export function register_addon(path: string): any;
/**
 * Register a sample with PCM data
 */
export function register_sample(uri: string, pcm: Float32Array): any;
/**
 * List registered banks
 */
export function list_registered_banks(): any;
/**
 * List registered samples
 */
export function list_registered_samples(limit: number): any;
/**
 * Get sample load events log
 */
export function sample_load_events(clear: boolean): any;
/**
 * Get playback debug log
 */
export function collect_playback_debug(clear: boolean): any;
/**
 * Get playback debug state
 */
export function playback_debug_state(): any;
/**
 * Enable/disable debug error logging
 */
export function set_wasm_debug_errors(enable: boolean): void;
/**
 * Get last errors (legacy string format)
 */
export function collect_last_errors(clear: boolean): any;
/**
 * Get structured parse errors with line/column information
 */
export function collect_parse_errors(clear: boolean): any;
/**
 * Register a bank from a simple JSON manifest (for testing/manual registration)
 *
 * Manifest format:
 * ```json
 * {
 *   "name": "devaloop.808",
 *   "alias": "kit",
 *   "version": "1.0.0",
 *   "description": "808 drum bank",
 *   "triggers": {
 *     "kick": "http://example.com/kick.wav",
 *     "snare": "http://example.com/snare.wav"
 *   }
 * }
 * ```
 *
 * This only registers the bank metadata. Samples must be registered separately using register_sample().
 */
export function register_bank_json(manifest_json: string): void;
/**
 * Load and register a complete bank from bank.toml hosted at base_url
 *
 * Steps:
 * 1. Fetch base_url + "/bank.toml"
 * 2. Parse triggers
 * 3. For each trigger.path => fetch WAV file (relative to base_url)
 * 4. Parse WAV directly in Rust (no Web Audio API needed)
 * 5. Register samples with URI: devalang://bank/{publisher.name}/{path}
 * 6. Call register_addon() with query string for triggers
 *
 * Returns: { ok, bank, base_url, triggers: [{ name, uri, relative, frames }] }
 */
export function register_bank_from_manifest(base_url: string): Promise<any>;
/**
 * Load a bank from a URL (auto-detects bank.toml or bank.json)
 *
 * Tries in order:
 * 1. Exact URL if it ends with .toml/.json
 * 2. {base_url}/bank.toml
 * 3. {base_url}/bank.json
 *
 * Example:
 * - `load_bank_from_url("https://example.com/banks/devaloop/808")`
 *   → tries "https://example.com/banks/devaloop/808/bank.toml" then "bank.json"
 * - `load_bank_from_url("https://example.com/banks/kit.bank.json")`
 *   → loads exactly that JSON file
 */
export function load_bank_from_url(url: string): Promise<any>;
/**
 * Render audio from Devalang code
 * Returns audio buffer as Float32Array
 */
export function render_audio(user_code: string, options: any): Float32Array;
/**
 * Render audio with debug information
 */
export function debug_render(user_code: string, options: any): any;
/**
 * Render WAV file preview
 */
export function render_wav_preview(user_code: string, options: any, on_progress?: Function | null): Uint8Array;
/**
 * Get code to buffer metadata with duration calculation
 */
export function get_code_to_buffer_metadata(user_code: string, options: any): any;
/**
 * Get metadata for code without full rendering (fast preview)
 */
export function get_render_metadata(user_code: string, options: any): any;
/**
 * Export audio with format options (WAV 16/24/32 bit, MP3)
 */
export function export_audio(user_code: string, options: any, on_progress?: Function | null): Uint8Array;
/**
 * Render MIDI from Devalang code
 * Returns MIDI file as Uint8Array
 */
export function render_midi_array(user_code: string, options: any, on_progress?: Function | null): Uint8Array;
/**
 * Export MIDI file (browser download)
 * This is just a convenience wrapper that returns the same data as render_midi_array
 */
export function export_midi_file(user_code: string, options: any, on_progress?: Function | null): Uint8Array;
/**
 * Parse Devalang source code
 *
 * # Arguments
 * * `entry_path` - Path to the source file (for error messages)
 * * `source` - Source code to parse
 *
 * # Returns
 * JSON object with parse results
 */
export function parse(entry_path: string, source: string): any;
/**
 * Quick parse check - returns true if code parses without errors
 */
export function check_syntax(source: string): boolean;

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
  readonly memory: WebAssembly.Memory;
  readonly is_hot_reload_enabled: () => number;
  readonly collect_playhead_events: () => [number, number, number];
  readonly register_addon: (a: number, b: number) => [number, number, number];
  readonly register_sample: (a: number, b: number, c: any) => [number, number, number];
  readonly list_registered_banks: () => [number, number, number];
  readonly list_registered_samples: (a: number) => [number, number, number];
  readonly sample_load_events: (a: number) => [number, number, number];
  readonly collect_playback_debug: (a: number) => [number, number, number];
  readonly playback_debug_state: () => [number, number, number];
  readonly set_wasm_debug_errors: (a: number) => void;
  readonly collect_last_errors: (a: number) => [number, number, number];
  readonly collect_parse_errors: (a: number) => [number, number, number];
  readonly enable_hot_reload: (a: any) => void;
  readonly disable_hot_reload: () => void;
  readonly register_playhead_callback: (a: any) => any;
  readonly register_bank_json: (a: number, b: number) => [number, number];
  readonly register_bank_from_manifest: (a: number, b: number) => any;
  readonly load_bank_from_url: (a: number, b: number) => any;
  readonly render_audio: (a: number, b: number, c: any) => [number, number, number];
  readonly debug_render: (a: number, b: number, c: any) => [number, number, number];
  readonly render_wav_preview: (a: number, b: number, c: any, d: number) => [number, number, number];
  readonly get_code_to_buffer_metadata: (a: number, b: number, c: any) => [number, number, number];
  readonly get_render_metadata: (a: number, b: number, c: any) => [number, number, number];
  readonly export_audio: (a: number, b: number, c: any, d: number) => [number, number, number];
  readonly export_midi_file: (a: number, b: number, c: any, d: number) => [number, number, number];
  readonly parse: (a: number, b: number, c: number, d: number) => [number, number, number];
  readonly check_syntax: (a: number, b: number) => number;
  readonly render_midi_array: (a: number, b: number, c: any, d: number) => [number, number, number];
  readonly __wbindgen_malloc: (a: number, b: number) => number;
  readonly __wbindgen_realloc: (a: number, b: number, c: number, d: number) => number;
  readonly __wbindgen_exn_store: (a: number) => void;
  readonly __externref_table_alloc: () => number;
  readonly __wbindgen_export_4: WebAssembly.Table;
  readonly __wbindgen_export_5: WebAssembly.Table;
  readonly __externref_table_dealloc: (a: number) => void;
  readonly wasm_bindgen__convert__closures_____invoke__h4a794a844ef21195: (a: number, b: number) => void;
  readonly closure1048_externref_shim: (a: number, b: number, c: any) => void;
  readonly closure1082_externref_shim: (a: number, b: number, c: any, d: any) => void;
  readonly __wbindgen_start: () => void;
}

export type SyncInitInput = BufferSource | WebAssembly.Module;
/**
* Instantiates the given `module`, which can either be bytes or
* a precompiled `WebAssembly.Module`.
*
* @param {{ module: SyncInitInput }} module - Passing `SyncInitInput` directly is deprecated.
*
* @returns {InitOutput}
*/
export function initSync(module: { module: SyncInitInput } | SyncInitInput): InitOutput;

/**
* If `module_or_path` is {RequestInfo} or {URL}, makes a request and
* for everything else, calls `WebAssembly.instantiate` directly.
*
* @param {{ module_or_path: InitInput | Promise<InitInput> }} module_or_path - Passing `InitInput` directly is deprecated.
*
* @returns {Promise<InitOutput>}
*/
export default function __wbg_init (module_or_path?: { module_or_path: InitInput | Promise<InitInput> } | InitInput | Promise<InitInput>): Promise<InitOutput>;
