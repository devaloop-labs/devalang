/**
 * High-level API wrapper for Devalang WASM functions
 *
 * This module provides TypeScript-friendly wrappers around the raw WASM bindings,
 * with proper error handling and type conversions.
 */
import type { RenderOptions, MidiOptions, ParseResult, DebugRenderResult, CodeMetadata, RegisteredBank, DebugState } from './types.js';
/**
 * Parse Devalang source code
 *
 * @param entryPath - Path to the source file (for error messages)
 * @param source - Devalang source code
 * @returns Parse result with statements or errors
 *
 * @example
 * ```typescript
 * const result = await parse('test.deva', 'bpm 120\nsleep 1b');
 * if (result.success) {
 *   console.log(`Parsed ${result.statements.length} statements`);
 * }
 * ```
 */
export declare function parse(entryPath: string, source: string): Promise<ParseResult>;
/**
 * Quick syntax check without full parsing
 *
 * @param source - Devalang source code
 * @returns true if syntax is valid
 *
 * @example
 * ```typescript
 * if (await checkSyntax('bpm 120')) {
 *   console.log('Valid syntax!');
 * }
 * ```
 */
export declare function checkSyntax(source: string): Promise<boolean>;
/**
 * Render audio from Devalang code
 *
 * @param code - Devalang source code
 * @param options - Render options (sample rate, BPM)
 * @returns Float32Array containing rendered audio samples
 *
 * @example
 * ```typescript
 * const audio = await renderAudio('bpm 120\nlet s = synth sine {}\ns -> note(A4, { duration: 500 })', {
 *   sampleRate: 44100,
 *   bpm: 120
 * });
 * console.log(`Rendered ${audio.length} samples`);
 * ```
 */
export declare function renderAudio(code: string, options?: RenderOptions): Promise<Float32Array>;
/**
 * Render audio with debug information
 *
 * @param code - Devalang source code
 * @param options - Render options
 * @returns Debug render result with audio and metadata
 *
 * @example
 * ```typescript
 * const result = await debugRender('bpm 120\nlet s = synth sine {}\ns -> note(C4, { duration: 500 })');
 * console.log(`Duration: ${result.duration}s, Events: ${result.eventCount}`);
 * ```
 */
export declare function debugRender(code: string, options?: RenderOptions): Promise<DebugRenderResult>;
/**
 * Render WAV file preview as bytes
 *
 * @param code - Devalang source code
 * @param options - Render options
 * @returns Uint8Array containing WAV file bytes
 *
 * @example
 * ```typescript
 * const wavBytes = await renderWavPreview('bpm 120\nsleep 1b');
 * // Save to file or send to browser for download
 * ```
 */
export declare function renderWavPreview(code: string, options?: RenderOptions): Promise<Uint8Array>;
/**
 * Render MIDI file as bytes
 *
 * @param code - Devalang source code
 * @param options - MIDI options
 * @returns Uint8Array containing MIDI file bytes
 *
 * @example
 * ```typescript
 * const midiBytes = await renderMidi('bpm 120\nlet s = synth sine {}\ns -> note(C4, { duration: 1000 })');
 * // Save as .mid file
 * ```
 */
export declare function renderMidi(code: string, options?: MidiOptions): Promise<Uint8Array>;
/**
 * Get code metadata without rendering
 *
 * @param code - Devalang source code
 * @param options - Options
 * @returns Metadata about the code
 *
 * @example
 * ```typescript
 * const meta = await getCodeMetadata('bpm 140\nsleep 1b\nsleep 1b');
 * console.log(`${meta.statementCount} statements at ${meta.bpm} BPM`);
 * ```
 */
export declare function getCodeMetadata(code: string, options?: RenderOptions): Promise<CodeMetadata>;
/**
 * Register an audio sample with PCM data
 *
 * @param uri - Sample URI (e.g., "devalang://bank/kick.wav")
 * @param pcm - PCM audio data as Float32Array
 *
 * @example
 * ```typescript
 * const pcm = new Float32Array([0.5, -0.5, 0.25, -0.25]);
 * await registerSample('devalang://bank/test.wav', pcm);
 * ```
 */
export declare function registerSample(uri: string, pcm: Float32Array): Promise<boolean>;
/**
 * Register an audio bank addon
 *
 * @param path - Addon path (e.g., "devalang://bank/publisher.name?triggers=kick:audio/kick.wav")
 *
 * @example
 * ```typescript
 * await registerAddon('devalang://bank/devaloop.808?triggers=kick:audio/kick.wav,snare:audio/snare.wav');
 * ```
 */
export declare function registerAddon(path: string): Promise<void>;
/**
 * List all registered banks
 *
 * @returns Array of registered banks
 *
 * @example
 * ```typescript
 * const banks = await listRegisteredBanks();
 * banks.forEach(bank => console.log(`${bank.alias}: ${bank.fullName}`));
 * ```
 */
export declare function listRegisteredBanks(): Promise<RegisteredBank[]>;
/**
 * Get sample load events log
 *
 * @param clear - Whether to clear the log after reading
 * @returns Array of log messages
 */
export declare function getSampleLoadEvents(clear?: boolean): Promise<string[]>;
/**
 * Get playback debug log
 *
 * @param clear - Whether to clear the log after reading
 * @returns Array of debug messages
 */
export declare function getPlaybackDebug(clear?: boolean): Promise<string[]>;
/**
 * Get debug state
 *
 * @returns Current debug state
 */
export declare function getDebugState(): Promise<DebugState>;
/**
 * Enable or disable debug error logging
 *
 * @param enable - Whether to enable debug errors
 */
export declare function setDebugErrors(enable: boolean): Promise<void>;
/**
 * Get last errors
 *
 * @param clear - Whether to clear errors after reading
 * @returns Array of error messages
 */
export declare function getLastErrors(clear?: boolean): Promise<string[]>;
//# sourceMappingURL=api.d.ts.map