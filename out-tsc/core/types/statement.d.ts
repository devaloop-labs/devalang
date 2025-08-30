import { Value, Duration } from "./value";
/**
 * Represents a statement kind in the code.
 */
export type StatementKind = {
    kind: "Tempo";
} | {
    kind: "Bank";
    alias?: string;
} | {
    kind: "Print";
} | {
    kind: "Load";
    source: string;
    alias: string;
} | {
    kind: "Use";
    name: string;
    alias?: string;
} | {
    kind: "Let";
    name: string;
} | {
    kind: "Automate";
    target: string;
} | {
    kind: "ArrowCall";
    target: string;
    method: string;
    args: Value[];
} | {
    kind: "Function";
    name: string;
    parameters: string[];
    body: Statement[];
} | {
    kind: "Synth";
} | {
    kind: "Trigger";
    entity: string;
    duration: Duration;
    effects?: Value;
} | {
    kind: "Sleep";
} | {
    kind: "Call";
    name: string;
    args: Value[];
} | {
    kind: "Spawn";
    name: string;
    args: Value[];
} | {
    kind: "Loop";
} | {
    kind: "Group";
} | {
    kind: "Include";
    path: string;
} | {
    kind: "Export";
    names: string[];
    source: string;
} | {
    kind: "Import";
    names: string[];
    source: string;
} | {
    kind: "If";
} | {
    kind: "Else";
} | {
    kind: "ElseIf";
} | {
    kind: "Comment";
} | {
    kind: "Indent";
} | {
    kind: "Dedent";
} | {
    kind: "NewLine";
} | {
    kind: "On";
    event: string;
    args?: Value[];
    body: Statement[];
} | {
    kind: "Emit";
    event: string;
    payload?: Value;
} | {
    kind: "Unknown";
} | {
    kind: "Error";
    message: string;
};
/**
 * Represents a statement in the code.
 */
export interface Statement {
    kind: StatementKind;
    value: Value;
    indent: number;
    line: number;
    column: number;
}
