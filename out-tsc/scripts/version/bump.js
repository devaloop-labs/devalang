"use strict";
var __createBinding = (this && this.__createBinding) || (Object.create ? (function(o, m, k, k2) {
    if (k2 === undefined) k2 = k;
    var desc = Object.getOwnPropertyDescriptor(m, k);
    if (!desc || ("get" in desc ? !m.__esModule : desc.writable || desc.configurable)) {
      desc = { enumerable: true, get: function() { return m[k]; } };
    }
    Object.defineProperty(o, k2, desc);
}) : (function(o, m, k, k2) {
    if (k2 === undefined) k2 = k;
    o[k2] = m[k];
}));
var __setModuleDefault = (this && this.__setModuleDefault) || (Object.create ? (function(o, v) {
    Object.defineProperty(o, "default", { enumerable: true, value: v });
}) : function(o, v) {
    o["default"] = v;
});
var __importStar = (this && this.__importStar) || (function () {
    var ownKeys = function(o) {
        ownKeys = Object.getOwnPropertyNames || function (o) {
            var ar = [];
            for (var k in o) if (Object.prototype.hasOwnProperty.call(o, k)) ar[ar.length] = k;
            return ar;
        };
        return ownKeys(o);
    };
    return function (mod) {
        if (mod && mod.__esModule) return mod;
        var result = {};
        if (mod != null) for (var k = ownKeys(mod), i = 0; i < k.length; i++) if (k[i] !== "default") __createBinding(result, mod, k[i]);
        __setModuleDefault(result, mod);
        return result;
    };
})();
Object.defineProperty(exports, "__esModule", { value: true });
exports.bumpVersion = bumpVersion;
const fs = __importStar(require("fs"));
const path = __importStar(require("path"));
const child_process_1 = require("child_process");
/**
 * Bumps the version across all project files
 */
async function bumpVersion(type, preTag = 'beta') {
    const rootPath = path.resolve(__dirname, '../../../..');
    // Read current versions
    const packageJsonPath = path.join(rootPath, 'package.json');
    const cargoTomlPath = path.join(rootPath, 'Cargo.toml');
    const projectVersionPath = path.join(rootPath, 'project-version.json');
    const packageJson = JSON.parse(fs.readFileSync(packageJsonPath, 'utf-8'));
    const currentVersion = packageJson.version;
    // Parse version
    const versionMatch = currentVersion.match(/^(\d+)\.(\d+)\.(\d+)(?:-(.+))?$/);
    if (!versionMatch) {
        throw new Error(`Invalid version format: ${currentVersion}`);
    }
    let [, major, minor, patch, pre] = versionMatch;
    let majorNum = parseInt(major);
    let minorNum = parseInt(minor);
    let patchNum = parseInt(patch);
    // Calculate new version
    let newVersion;
    let channel = 'stable';
    switch (type) {
        case 'major':
            majorNum++;
            minorNum = 0;
            patchNum = 0;
            newVersion = `${majorNum}.${minorNum}.${patchNum}`;
            break;
        case 'minor':
            minorNum++;
            patchNum = 0;
            newVersion = `${majorNum}.${minorNum}.${patchNum}`;
            break;
        case 'patch':
            patchNum++;
            newVersion = `${majorNum}.${minorNum}.${patchNum}`;
            break;
        case 'pre':
            const preVersion = pre ? parseInt(pre.split('.')[1] || '0') + 1 : 0;
            newVersion = `${majorNum}.${minorNum}.${patchNum}-${preTag}.${preVersion}`;
            channel = preTag;
            break;
    }
    // Get git commit hash
    let commit;
    try {
        commit = (0, child_process_1.execSync)('git rev-parse --short HEAD', { encoding: 'utf-8' }).trim();
    }
    catch (error) {
        console.warn('⚠️  Could not get git commit hash');
    }
    // Update package.json
    packageJson.version = newVersion;
    fs.writeFileSync(packageJsonPath, JSON.stringify(packageJson, null, 2) + '\n');
    console.log(`✅ Updated package.json to ${newVersion}`);
    // Update Cargo.toml
    let cargoToml = fs.readFileSync(cargoTomlPath, 'utf-8');
    cargoToml = cargoToml.replace(/version = "[\d\.]+-?[a-z\.]*\d*"/, `version = "${newVersion}"`);
    fs.writeFileSync(cargoTomlPath, cargoToml);
    console.log(`✅ Updated Cargo.toml to ${newVersion}`);
    // Update project-version.json
    const projectVersion = {
        version: newVersion,
        channel,
        build: 1, // Reset build number on version bump
        commit
    };
    fs.writeFileSync(projectVersionPath, JSON.stringify(projectVersion, null, 2) + '\n');
    console.log(`✅ Updated project-version.json to ${newVersion}`);
    // Git commit
    try {
        (0, child_process_1.execSync)(`git add ${packageJsonPath} ${cargoTomlPath} ${projectVersionPath}`, { stdio: 'inherit' });
        (0, child_process_1.execSync)(`git commit -m "chore: bump version to ${newVersion}"`, { stdio: 'inherit' });
        console.log(`✅ Created git commit for version ${newVersion}`);
    }
    catch (error) {
        console.warn('⚠️  Could not create git commit (you may need to commit manually)');
    }
    return newVersion;
}
//# sourceMappingURL=bump.js.map