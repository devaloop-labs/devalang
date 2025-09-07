import { ParseResult } from "../types/result";
import { DebugResult } from "../types/result";
/**
 * Parses the user code.
 * @param entry_path The entry path for the code.
 * @param source The source code to parse.
 * @returns ParseResult | any
 */
export declare function parse(entry_path: string, source: string): ParseResult | any;
/**
 * Renders the debug information for the user code.
 * @param user_code The user-provided code to render debug information for.
 * @returns DebugResult | any
 */
export declare function debug_render(user_code: string): DebugResult | any;
/**
 * Renders audio from the user code.
 * @param user_code The user-provided code to render audio from.
 * @returns Float32Array
 */
export declare function render_audio(user_code: string): Float32Array;
/**
 * Register a JS callback to receive playhead events { time, line, column } during playback.
 * The callback will be called with a single object argument.
 * @param cb The callback function to register.
 * @returns void
 */
export declare function register_playhead_callback(cb: (ev: {
    time: number;
    line: number;
    column: number;
}) => void): any;
/**
 * Collects playhead events that have been recorded during playback.
 * @returns Array of playhead events { time, line, column }.
 */
export declare function collect_playhead_events(): any;
/**
 * Unregisters the JS callback for playhead events.
 * @returns void
 */
export declare function unregister_playhead_callback(): any;
