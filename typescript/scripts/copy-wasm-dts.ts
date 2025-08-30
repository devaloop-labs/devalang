import * as fs from 'fs';
import * as path from 'path';

// Copies .d.ts files from pkg/ (wasm-pack output) into out-tsc/pkg/
const ROOT = path.resolve(__dirname, '..', '..');
const PKG_DIR = path.join(ROOT, 'pkg');
const OUT_DIR = path.join(ROOT, 'out-tsc', 'pkg');

function ensureDir(p: string): void {
  if (!fs.existsSync(p)) fs.mkdirSync(p, { recursive: true });
}

function copyDir(src: string, dest: string): void {
  ensureDir(dest);
  const items = fs.readdirSync(src, { withFileTypes: true });
  for (const item of items) {
    const srcPath = path.join(src, item.name);
    const destPath = path.join(dest, item.name);
    if (item.isDirectory()) {
      copyDir(srcPath, destPath);
    } else if (item.isFile() && item.name.endsWith('.d.ts')) {
      fs.copyFileSync(srcPath, destPath);
      console.log('copied', srcPath, '->', destPath);
    }
  }
}

function main(): number {
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
if (exitCode !== 0) process.exit(exitCode);
