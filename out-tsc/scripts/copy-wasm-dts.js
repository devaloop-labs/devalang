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
const fs = __importStar(require("fs"));
const path = __importStar(require("path"));
// Copies .d.ts files from pkg/ (wasm-pack output) into out-tsc/pkg/
const ROOT = path.resolve(__dirname, '..', '..');
const PKG_DIR = path.join(ROOT, 'pkg');
const OUT_DIR = path.join(ROOT, 'out-tsc', 'pkg');
function ensureDir(p) {
    if (!fs.existsSync(p))
        fs.mkdirSync(p, { recursive: true });
}
function copyDir(src, dest) {
    ensureDir(dest);
    const items = fs.readdirSync(src, { withFileTypes: true });
    for (const item of items) {
        const srcPath = path.join(src, item.name);
        const destPath = path.join(dest, item.name);
        if (item.isDirectory()) {
            copyDir(srcPath, destPath);
        }
        else if (item.isFile() && item.name.endsWith('.d.ts')) {
            fs.copyFileSync(srcPath, destPath);
            console.log('copied', srcPath, '->', destPath);
        }
    }
}
function main() {
    if (!fs.existsSync(PKG_DIR)) {
        console.error('pkg directory not found at', PKG_DIR);
        return 1;
    }
    ensureDir(OUT_DIR);
    copyDir(PKG_DIR, OUT_DIR);
    console.log('done');
    return 0;
}
const exitCode = main();
if (exitCode !== 0)
    process.exit(exitCode);
