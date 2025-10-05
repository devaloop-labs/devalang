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
exports.syncVersion = syncVersion;
const fs = __importStar(require("fs"));
const path = __importStar(require("path"));
/**
 * Synchronizes version from package.json to all other version files
 */
async function syncVersion() {
    const rootPath = path.resolve(__dirname, '../../../..');
    // Read package.json as source of truth
    const packageJsonPath = path.join(rootPath, 'package.json');
    const packageJson = JSON.parse(fs.readFileSync(packageJsonPath, 'utf-8'));
    const version = packageJson.version;
    if (!version) {
        throw new Error('No version found in package.json');
    }
    // Determine channel from version
    const isPreRelease = version.includes('-');
    let channel = 'stable';
    if (isPreRelease) {
        if (version.includes('alpha'))
            channel = 'alpha';
        else if (version.includes('beta'))
            channel = 'beta';
        else
            channel = 'preview';
    }
    // Update Cargo.toml
    const cargoTomlPath = path.join(rootPath, 'Cargo.toml');
    let cargoToml = fs.readFileSync(cargoTomlPath, 'utf-8');
    cargoToml = cargoToml.replace(/version = "[\d\.]+-?[a-z\.]*\d*"/, `version = "${version}"`);
    fs.writeFileSync(cargoTomlPath, cargoToml);
    console.log(`✅ Synchronized Cargo.toml to ${version}`);
    // Update project-version.json
    const projectVersionPath = path.join(rootPath, 'project-version.json');
    let projectVersion = { version, channel, build: 1 };
    if (fs.existsSync(projectVersionPath)) {
        const existing = JSON.parse(fs.readFileSync(projectVersionPath, 'utf-8'));
        projectVersion.build = existing.build || 1;
        projectVersion.commit = existing.commit;
    }
    projectVersion.version = version;
    projectVersion.channel = channel;
    fs.writeFileSync(projectVersionPath, JSON.stringify(projectVersion, null, 2) + '\n');
    console.log(`✅ Synchronized project-version.json to ${version}`);
    // Update installer/windows/devalang.wxs (WiX installer)
    const wixFilePath = path.join(rootPath, 'installer', 'windows', 'devalang.wxs');
    if (fs.existsSync(wixFilePath)) {
        try {
            let wixContent = fs.readFileSync(wixFilePath, 'utf-8');
            // Update ProductVersion in the <?define ?> section
            wixContent = wixContent.replace(/<\?define ProductVersion="[\d\.]+\-?[a-z\.]*\d*" \?>/, `<` + `?define ProductVersion="${version}" ?` + `>`);
            fs.writeFileSync(wixFilePath, wixContent);
            console.log(`✅ Synchronized installer/windows/devalang.wxs to ${version}`);
        }
        catch (error) {
            console.warn('⚠️  Could not update installer/windows/devalang.wxs:', error);
        }
    }
    else {
        console.warn('⚠️  installer/windows/devalang.wxs not found, skipping');
    }
}
//# sourceMappingURL=sync.js.map