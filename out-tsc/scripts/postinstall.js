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
const version = fs_2.default.readFileSync(projectVersionPath, "utf-8").trim();
const versionString = JSON.parse(version).version;
const platform = process.platform;
let binaryName;
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
    default:
        console.error(`❌ Unsupported platform: ${platform}`);
        process.exit(1);
}
const destDir = (0, path_1.join)(__dirname, "..", "..", "out-tsc", "bin");
const dest = (0, path_1.join)(destDir, binaryName);
const url = `https://github.com/devaloop-labs/devalang/releases/download/v${versionString}/${binaryName}`;
(0, fs_1.mkdirSync)(destDir, { recursive: true });
console.log(`⬇️  Downloading ${binaryName} from ${url}`);
follow_redirects_1.https.get(url, (res) => {
    if (res.statusCode !== 200) {
        console.error(`❌ Failed (HTTP ${res.statusCode})`);
        process.exit(1);
    }
    const file = (0, fs_1.createWriteStream)(dest, { mode: 0o755 });
    res.pipe(file);
    file.on("finish", () => {
        file.close();
        console.log(`✅ Downloaded ${binaryName} to ${dest}`);
    });
}).on("error", (err) => {
    console.error(`❌ Error: ${err.message}`);
    process.exit(1);
});
