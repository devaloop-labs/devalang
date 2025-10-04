/**
 * High-level API wrapper for Devalang WASM functions
 * 
 * This module provides TypeScript-friendly wrappers around the raw WASM bindings,
 * with proper error handling and type conversions.
 */

import type {
  RenderOptions,
  MidiOptions,
  ParseResult,
  DebugRenderResult,
  CodeMetadata,
  RegisteredBank,
  DebugState
} from './types.js';

// WASM module will be loaded dynamically
let wasmModule: any = null;

/**
 * Initialize the WASM module
 * This is called automatically on first use
 */
async function ensureWasmLoaded(): Promise<void> {
  if (wasmModule) return;
  
  try {
    // Try to load from different possible locations
    try {
      wasmModule = await import('../../pkg/web/devalang_wasm.js');
    } catch {
      wasmModule = await import('../../pkg/node/devalang_wasm.js');
    }
  } catch (error) {
    throw new Error(
      'Failed to load WASM module. Make sure to run "npm run rust:wasm:all" first.\n' +
      `Error: ${error}`
    );
  }
}

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
export async function parse(entryPath: string, source: string): Promise<ParseResult> {
  await ensureWasmLoaded();
  return wasmModule.parse(entryPath, source);
}

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
export async function checkSyntax(source: string): Promise<boolean> {
  await ensureWasmLoaded();
  return wasmModule.check_syntax(source);
}

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
export async function renderAudio(code: string, options?: RenderOptions): Promise<Float32Array> {
  await ensureWasmLoaded();
  const opts = {
    sampleRate: options?.sampleRate ?? 44100,
    bpm: options?.bpm ?? 120
  };
  return wasmModule.render_audio(code, opts);
}

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
export async function debugRender(code: string, options?: RenderOptions): Promise<DebugRenderResult> {
  await ensureWasmLoaded();
  const opts = {
    sampleRate: options?.sampleRate ?? 44100,
    bpm: options?.bpm ?? 120
  };
  return wasmModule.debug_render(code, opts);
}

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
export async function renderWavPreview(code: string, options?: RenderOptions): Promise<Uint8Array> {
  await ensureWasmLoaded();
  const opts = {
    sampleRate: options?.sampleRate ?? 44100,
    bpm: options?.bpm ?? 120
  };
  return wasmModule.render_wav_preview(code, opts);
}

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
export async function renderMidi(code: string, options?: MidiOptions): Promise<Uint8Array> {
  await ensureWasmLoaded();
  const opts = {
    sampleRate: options?.sampleRate ?? 44100,
    bpm: options?.bpm ?? 120,
    timeSignatureNum: options?.timeSignatureNum ?? 4,
    timeSignatureDen: options?.timeSignatureDen ?? 4
  };
  return wasmModule.render_midi_array(code, opts);
}

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
export async function getCodeMetadata(code: string, options?: RenderOptions): Promise<CodeMetadata> {
  await ensureWasmLoaded();
  const opts = {
    sampleRate: options?.sampleRate ?? 44100,
    bpm: options?.bpm ?? 120
  };
  return wasmModule.get_code_to_buffer_metadata(code, opts);
}

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
export async function registerSample(uri: string, pcm: Float32Array): Promise<boolean> {
  await ensureWasmLoaded();
  return wasmModule.register_sample(uri, pcm);
}

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
export async function registerAddon(path: string): Promise<void> {
  await ensureWasmLoaded();
  return wasmModule.register_addon(path);
}

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
export async function listRegisteredBanks(): Promise<RegisteredBank[]> {
  await ensureWasmLoaded();
  return wasmModule.list_registered_banks();
}

/**
 * Get sample load events log
 * 
 * @param clear - Whether to clear the log after reading
 * @returns Array of log messages
 */
export async function getSampleLoadEvents(clear: boolean = false): Promise<string[]> {
  await ensureWasmLoaded();
  return wasmModule.sample_load_events(clear);
}

/**
 * Get playback debug log
 * 
 * @param clear - Whether to clear the log after reading
 * @returns Array of debug messages
 */
export async function getPlaybackDebug(clear: boolean = false): Promise<string[]> {
  await ensureWasmLoaded();
  return wasmModule.collect_playback_debug(clear);
}

/**
 * Get debug state
 * 
 * @returns Current debug state
 */
export async function getDebugState(): Promise<DebugState> {
  await ensureWasmLoaded();
  return wasmModule.playback_debug_state();
}

/**
 * Enable or disable debug error logging
 * 
 * @param enable - Whether to enable debug errors
 */
export async function setDebugErrors(enable: boolean): Promise<void> {
  await ensureWasmLoaded();
  wasmModule.set_wasm_debug_errors(enable);
}

/**
 * Get last errors
 * 
 * @param clear - Whether to clear errors after reading
 * @returns Array of error messages
 */
export async function getLastErrors(clear: boolean = false): Promise<string[]> {
  await ensureWasmLoaded();
  return wasmModule.collect_last_errors(clear);
}
