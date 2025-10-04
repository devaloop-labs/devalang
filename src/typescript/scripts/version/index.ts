#!/usr/bin/env node

/**
 * Version management CLI
 * Bumps version across package.json, Cargo.toml, and project-version.json
 */

import { bumpVersion } from './bump';
import { syncVersion } from './sync';
import { fetchVersion } from './fetch';

const args = process.argv.slice(2);
const command = args[0];
const options = args.slice(1);

async function main() {
  try {
    switch (command) {
      case 'bump': {
        const type = options[0] as 'major' | 'minor' | 'patch' | 'pre';
        if (!['major', 'minor', 'patch', 'pre'].includes(type)) {
          console.error('❌ Usage: npm run version:bump <major|minor|patch|pre>');
          process.exit(1);
        }
        const preTag = options[1] || 'beta';
        await bumpVersion(type, preTag);
        console.log(`✅ Version bumped to ${type}`);
        break;
      }

      case 'sync': {
        await syncVersion();
        console.log('✅ Version synchronized across all files');
        break;
      }

      case 'fetch': {
        const version = await fetchVersion();
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
  } catch (error) {
    console.error('❌ Error:', error);
    process.exit(1);
  }
}

main();
