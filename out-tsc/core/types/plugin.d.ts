/**
 * Represents a plugin export.
 */
export interface PluginExport {
    name: string;
    kind: string;
    default?: any;
}
/**
 * Represents a plugin.
 */
export interface PluginInfo {
    author: string;
    name: string;
    version?: string;
    description?: string;
    exports: PluginExport[];
}
