/* tslint:disable */
/* eslint-disable */
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
 * Get metadata for code without full rendering (fast preview)
 */
export function get_render_metadata(user_code: string, options: any): any;
/**
 * Export audio with format options (WAV 16/24/32 bit, MP3)
 */
export function export_audio(user_code: string, options: any, on_progress?: Function | null): Uint8Array;
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
 * Render MIDI from Devalang code
 * Returns MIDI file as Uint8Array
 */
export function render_midi_array(user_code: string, options: any, on_progress?: Function | null): Uint8Array;
/**
 * Export MIDI file (browser download)
 * This is just a convenience wrapper that returns the same data as render_midi_array
 */
export function export_midi_file(user_code: string, options: any, on_progress?: Function | null): Uint8Array;
