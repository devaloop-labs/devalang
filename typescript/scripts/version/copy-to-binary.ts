import fs from "fs";
import path from "path";

const argv = process.argv.slice(2);
let sourceArg: string | undefined;
let binaryArg: string | undefined;
let outDirArg: string | undefined;

for (let i = 0; i < argv.length; i++) {
  const a = argv[i];
  if (a === "--source") {
    sourceArg = argv[++i];
  } else if (a === "--binary") {
    binaryArg = argv[++i];
  } else if (a === "--out-dir") {
    outDirArg = argv[++i];
  } else if (a === "--help" || a === "-h") {
    process.exit(0);
  } else {
    console.error(`Unknown arg: ${a}`);
    process.exit(1);
  }
}

// Default source: attempt to locate project-version.json at repo root
const defaultSource = path.resolve(
  __dirname,
  "..",
  "..",
  "..",
  "project-version.json"
);
const sourcePath = sourceArg ? path.resolve(sourceArg) : defaultSource;

if (!fs.existsSync(sourcePath)) {
  console.error(
    `Source project-version.json not found at '${sourcePath}'. Please provide --source or ensure file exists.`
  );
  process.exit(2);
}

let destDir: string | null = null;

if (binaryArg) {
  const binPath = path.resolve(binaryArg);
  // If it's an existing file, use its directory, otherwise assume user passed target path and use its parent
  if (fs.existsSync(binPath) && fs.statSync(binPath).isFile()) {
    destDir = path.dirname(binPath);
  } else {
    // If binPath looks like a file path (has extension) use parent, else treat as dir
    const ext = path.extname(binPath);
    if (ext) {
      destDir = path.dirname(binPath);
    } else {
      destDir = binPath;
    }
  }
} else if (outDirArg) {
  destDir = path.resolve(outDirArg);
} else {
  // Default: try to copy next to the running node current working dir (useful when packaging)
  destDir = process.cwd();
}

if (!destDir) {
  console.error("Could not resolve destination directory");
  process.exit(3);
}

try {
  if (!fs.existsSync(destDir)) {
    fs.mkdirSync(destDir, { recursive: true });
  }

  const destPath = path.join(destDir, "project-version.json");
  fs.copyFileSync(sourcePath, destPath);
  console.log(`project-version.json copied to '${destPath}'`);
  process.exit(0);
} catch (err) {
  console.error("Failed to copy project-version.json:", err);
  process.exit(4);
}
