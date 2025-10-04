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
exports.fetchVersion = fetchVersion;
const fs = __importStar(require("fs"));
const path = __importStar(require("path"));
const child_process_1 = require("child_process");
/**
 * Fetches current version information and increments build number
 */
async function fetchVersion() {
    const rootPath = path.resolve(__dirname, '../../../..');
    const projectVersionPath = path.join(rootPath, 'project-version.json');
    let projectVersion;
    if (fs.existsSync(projectVersionPath)) {
        projectVersion = JSON.parse(fs.readFileSync(projectVersionPath, 'utf-8'));
    }
    else {
        // Fallback to package.json
        const packageJsonPath = path.join(rootPath, 'package.json');
        const packageJson = JSON.parse(fs.readFileSync(packageJsonPath, 'utf-8'));
        projectVersion = {
            version: packageJson.version,
            channel: 'stable',
            build: 1
        };
    }
    // Get git commit hash
    try {
        const commit = (0, child_process_1.execSync)('git rev-parse --short HEAD', { encoding: 'utf-8' }).trim();
        projectVersion.commit = commit;
    }
    catch (error) {
        console.warn('⚠️  Could not get git commit hash');
    }
    // Increment build number
    projectVersion.build++;
    // Save updated version
    fs.writeFileSync(projectVersionPath, JSON.stringify(projectVersion, null, 2) + '\n');
    return projectVersion;
}
//# sourceMappingURL=fetch.js.map