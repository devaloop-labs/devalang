/**
 * Copy WASM TypeScript definitions
 * 
 * This script copies the generated .d.ts files from wasm-pack build
 * to a more accessible location for TypeScript consumption.
 */

import * as fs from 'fs';
import * as path from 'path';

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
