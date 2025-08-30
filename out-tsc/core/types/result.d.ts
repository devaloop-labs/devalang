/**
 * Represents an error that occurred during parsing.
 */
export interface ErrorResult {
    message: string;
    line: number;
    column: number;
}
/**
 * Represents the result of parsing user code.
 */
export interface ParseResult {
    ok: boolean;
    ast: string;
    errors: ErrorResult[];
}
/**
 * Represents the result of debugging user code.
 */
export interface DebugResult {
    samples_len: number;
    any_nonzero: boolean;
    ast: string;
    note_count: number;
    global_vars: string[];
    statements_count: number;
}
