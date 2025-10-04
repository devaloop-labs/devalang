/**
 * Type definitions for Devalang V2
 */
/**
 * Options for rendering audio
 */
export interface RenderOptions {
    sampleRate?: number;
    bpm?: number;
}
/**
 * Options for MIDI export
 */
export interface MidiOptions extends RenderOptions {
    timeSignatureNum?: number;
    timeSignatureDen?: number;
}
/**
 * Parse result from Devalang parser
 */
export interface ParseResult {
    success: boolean;
    statements: StatementInfo[];
    errors: string[];
}
/**
 * Statement information
 */
export interface StatementInfo {
    kind: string;
    line: number;
}
/**
 * Debug render result with metadata
 */
export interface DebugRenderResult {
    audio: Float32Array;
    sampleRate: number;
    duration: number;
    eventCount: number;
    bpm: number;
}
/**
 * Metadata about code without rendering
 */
export interface CodeMetadata {
    statementCount: number;
    bpm: number;
    sampleRate: number;
}
/**
 * Registered bank information
 */
export interface RegisteredBank {
    fullName: string;
    alias: string;
    triggers: Record<string, string>;
}
/**
 * Debug state information
 */
export interface DebugState {
    sampleLoadCount: number;
    playbackDebugCount: number;
    errorCount: number;
    debugErrorsEnabled: boolean;
}
//# sourceMappingURL=types.d.ts.map