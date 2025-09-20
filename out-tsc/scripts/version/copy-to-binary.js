"use strict";
var __importDefault = (this && this.__importDefault) || function (mod) {
    return (mod && mod.__esModule) ? mod : { "default": mod };
};
Object.defineProperty(exports, "__esModule", { value: true });
const fs_1 = __importDefault(require("fs"));
const path_1 = __importDefault(require("path"));
const argv = process.argv.slice(2);
let sourceArg;
let binaryArg;
let outDirArg;
for (let i = 0; i < argv.length; i++) {
    const a = argv[i];
    if (a === "--source") {
        sourceArg = argv[++i];
    }
    else if (a === "--binary") {
        binaryArg = argv[++i];
    }
    else if (a === "--out-dir") {
        outDirArg = argv[++i];
    }
    else if (a === "--help" || a === "-h") {
        process.exit(0);
    }
    else {
        console.error(`Unknown arg: ${a}`);
        process.exit(1);
    }
}
// Default source: attempt to locate project-version.json at repo root
const defaultSource = path_1.default.resolve(__dirname, "..", "..", "..", "project-version.json");
const sourcePath = sourceArg ? path_1.default.resolve(sourceArg) : defaultSource;
if (!fs_1.default.existsSync(sourcePath)) {
    console.error(`Source project-version.json not found at '${sourcePath}'. Please provide --source or ensure file exists.`);
    process.exit(2);
}
let destDir = null;
if (binaryArg) {
    const binPath = path_1.default.resolve(binaryArg);
    // If it's an existing file, use its directory, otherwise assume user passed target path and use its parent
    if (fs_1.default.existsSync(binPath) && fs_1.default.statSync(binPath).isFile()) {
        destDir = path_1.default.dirname(binPath);
    }
    else {
        // If binPath looks like a file path (has extension) use parent, else treat as dir
        const ext = path_1.default.extname(binPath);
        if (ext) {
            destDir = path_1.default.dirname(binPath);
        }
        else {
            destDir = binPath;
        }
    }
}
else if (outDirArg) {
    destDir = path_1.default.resolve(outDirArg);
}
else {
    // Default: try to copy next to the running node current working dir (useful when packaging)
    destDir = process.cwd();
}
if (!destDir) {
    console.error("Could not resolve destination directory");
    process.exit(3);
}
try {
    if (!fs_1.default.existsSync(destDir)) {
        fs_1.default.mkdirSync(destDir, { recursive: true });
    }
    const destPath = path_1.default.join(destDir, "project-version.json");
    fs_1.default.copyFileSync(sourcePath, destPath);
    console.log(`project-version.json copied to '${destPath}'`);
    process.exit(0);
}
catch (err) {
    console.error("Failed to copy project-version.json:", err);
    process.exit(4);
}
