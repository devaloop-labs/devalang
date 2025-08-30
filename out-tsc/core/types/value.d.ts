import { Statement } from "./statement";
/**
 * Represents a duration in the code.
 */
export type Duration = {
    Number: number;
} | {
    Identifier: string;
} | {
    Beat: string;
} | {
    Auto: null;
};
/**
 * Represents a value in the code.
 */
export type Value = {
    Boolean: boolean;
} | {
    Number: number;
} | {
    Duration: Duration;
} | {
    Identifier: string;
} | {
    String: string;
} | {
    Array: Value[];
} | {
    Map: Record<string, Value>;
} | {
    Block: Statement[];
} | {
    Sample: string;
} | {
    Beat: string;
} | {
    Statement: Statement;
} | {
    StatementKind: any;
} | {
    Unknown: null;
} | null;
