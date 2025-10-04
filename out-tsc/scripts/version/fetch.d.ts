interface ProjectVersion {
    version: string;
    channel: 'stable' | 'preview' | 'beta' | 'alpha';
    build: number;
    commit?: string;
}
/**
 * Fetches current version information and increments build number
 */
export declare function fetchVersion(): Promise<ProjectVersion>;
export {};
//# sourceMappingURL=fetch.d.ts.map