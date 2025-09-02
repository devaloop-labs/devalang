"use strict";
var __importDefault = (this && this.__importDefault) || function (mod) {
    return (mod && mod.__esModule) ? mod : { "default": mod };
};
Object.defineProperty(exports, "__esModule", { value: true });
const fs_1 = require("fs");
const path_1 = require("path");
const follow_redirects_1 = require("follow-redirects");
const fs_2 = __importDefault(require("fs"));
const path_2 = __importDefault(require("path"));
const projectVersionPath = path_2.default.join(__dirname, "../../project-version.json");
let versionString = "";
try {
    if (!fs_2.default.existsSync(projectVersionPath)) {
        console.warn(`⚠️  project-version.json not found at ${projectVersionPath}. Skipping postinstall binary download.`);
    }
    else {
        const version = fs_2.default.readFileSync(projectVersionPath, "utf-8").trim();
        try {
            versionString = JSON.parse(version).version;
        }
        catch (e) {
            console.warn(`⚠️  Failed to parse project-version.json: ${e.message}. Skipping binary download.`);
            versionString = "";
        }
    }
}
catch (e) {
    console.warn(`⚠️  Could not read project-version.json: ${e.message}. Skipping binary download.`);
    versionString = "";
}
const platform = process.platform;
let binaryName = "";
switch (platform) {
    case "win32":
        binaryName = "devalang-x86_64-pc-windows-msvc.exe";
        break;
    case "darwin":
        binaryName = "devalang-x86_64-apple-darwin";
        break;
    case "linux":
        binaryName = "devalang-x86_64-unknown-linux-gnu";
        break;
}
if (binaryName !== "" && versionString) {
    const destDir = (0, path_1.join)(__dirname, "..", "..", "out-tsc", "bin");
    const dest = (0, path_1.join)(destDir, binaryName);
    const url = `https://github.com/devaloop-labs/devalang/releases/download/v${versionString}/${binaryName}`;
    (0, fs_1.mkdirSync)(destDir, { recursive: true });
    console.log(`⬇️  Downloading ${binaryName} from ${url}`);
    const req = follow_redirects_1.https.get(url, (res) => {
        if (res.statusCode === 404) {
            console.warn(`⚠️  Asset not found (HTTP 404). Skipping binary download.`);
            res.resume();
            return;
        }
        if (res.statusCode !== 200) {
            console.warn(`⚠️  Failed (HTTP ${res.statusCode}). Skipping binary download.`);
            res.resume();
            return;
        }
        const file = (0, fs_1.createWriteStream)(dest, { mode: 0o755 });
        res.pipe(file);
        file.on("finish", () => {
            file.close();
            console.log(`✅ Downloaded ${binaryName} to ${dest}`);
        });
    });
    req.setTimeout(30000, () => {
        console.warn(`⚠️  Download timed out after 30s. Skipping binary download for ${binaryName}.`);
        try {
            req.destroy();
        }
        catch (e) { }
    });
    req.on("error", (err) => {
        // Network or other errors should not fail CI; log and continue
        console.warn(`⚠️  Error downloading binary: ${err.message}. Skipping binary download.`);
    });
}
else {
    console.warn(`⚠️  Unsupported platform: ${platform}. Skipping binary download.`);
}
