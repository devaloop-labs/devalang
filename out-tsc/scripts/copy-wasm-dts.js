"use strict";
/**
 * Copy WASM TypeScript definitions
 *
 * This script copies the generated .d.ts files from wasm-pack build
 * to a more accessible location for TypeScript consumption.
 */
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
const PKG_DIRS = ['pkg/web', 'pkg/node'];
const OUTPUT_DIR = 'typescript/generated';
function copyWasmDts() {
    console.log('📦 Copying WASM TypeScript definitions...\n');
    // Create output directory
    if (!fs.existsSync(OUTPUT_DIR)) {
        fs.mkdirSync(OUTPUT_DIR, { recursive: true });
    }
    for (const pkgDir of PKG_DIRS) {
        const pkgPath = path.join(process.cwd(), pkgDir);
        if (!fs.existsSync(pkgPath)) {
            console.log(`⚠️  ${pkgDir} not found - skipping`);
            continue;
        }
        // Find .d.ts files
        const files = fs.readdirSync(pkgPath);
        const dtsFiles = files.filter(f => f.endsWith('.d.ts'));
        if (dtsFiles.length === 0) {
            console.log(`⚠️  No .d.ts files found in ${pkgDir}`);
            continue;
        }
        for (const file of dtsFiles) {
            const sourcePath = path.join(pkgPath, file);
            const targetName = file.replace('.d.ts', `_${path.basename(pkgDir)}.d.ts`);
            const targetPath = path.join(OUTPUT_DIR, targetName);
            fs.copyFileSync(sourcePath, targetPath);
            console.log(`✅ Copied ${file} → ${targetName}`);
        }
    }
    console.log('\n✨ Done!\n');
}
// Run
copyWasmDts();
//# sourceMappingURL=copy-wasm-dts.js.map