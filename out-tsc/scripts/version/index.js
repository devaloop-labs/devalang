#!/usr/bin/env node
"use strict";
/**
 * Version management CLI
 * Bumps version across package.json, Cargo.toml, and project-version.json
 */
Object.defineProperty(exports, "__esModule", { value: true });
const bump_1 = require("./bump");
const sync_1 = require("./sync");
const fetch_1 = require("./fetch");
const args = process.argv.slice(2);
const command = args[0];
const options = args.slice(1);
async function main() {
    try {
        switch (command) {
            case 'bump': {
                const type = options[0];
                if (!['major', 'minor', 'patch', 'pre'].includes(type)) {
                    console.error('❌ Usage: npm run version:bump <major|minor|patch|pre>');
                    process.exit(1);
                }
                const preTag = options[1] || 'beta';
                await (0, bump_1.bumpVersion)(type, preTag);
                console.log(`✅ Version bumped to ${type}`);
                break;
            }
            case 'sync': {
                await (0, sync_1.syncVersion)();
                console.log('✅ Version synchronized across all files');
                break;
            }
            case 'fetch': {
                const version = await (0, fetch_1.fetchVersion)();
                console.log(`📦 Current version: ${version.version}`);
                console.log(`📍 Channel: ${version.channel}`);
                console.log(`🏗️  Build: ${version.build}`);
                console.log(`🔖 Commit: ${version.commit || 'unknown'}`);
                break;
            }
            default:
                console.error('❌ Usage: npm run version:bump <major|minor|patch|pre> [pre-tag]');
                console.error('       npm run version:sync');
                console.error('       npm run version:fetch');
                process.exit(1);
        }
    }
    catch (error) {
        console.error('❌ Error:', error);
        process.exit(1);
    }
}
main();
//# sourceMappingURL=index.js.map