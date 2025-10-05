import { createWriteStream, mkdirSync } from "fs";
import { join } from "path";
import { https } from "follow-redirects";
import fs from "fs";
import path from "path";

// From out-tsc/scripts/ we need to go up 2 levels to reach project root
const projectVersionPath = path.join(__dirname, "../../project-version.json");
let versionString = "";
try {
  if (!fs.existsSync(projectVersionPath)) {
    console.warn(`⚠️  project-version.json not found at ${projectVersionPath}. Skipping postinstall binary download.`);
  } else {
    const version = fs.readFileSync(projectVersionPath, "utf-8").trim();
    try {
      versionString = JSON.parse(version).version;
    } catch (e: any) {
      console.warn(`⚠️  Failed to parse project-version.json: ${e.message}. Skipping binary download.`);
      versionString = "";
    }
  }
} catch (e: any) {
  console.warn(`⚠️  Could not read project-version.json: ${e.message}. Skipping binary download.`);
  versionString = "";
}

const platform = process.platform;

let binaryName: string = "";

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
  // From out-tsc/scripts/ we need to go up 2 levels to reach project root
  const destDir = join(__dirname, "../../out-tsc/bin");
  const dest = join(destDir, binaryName);
  const url = `https://github.com/devaloop-labs/devalang/releases/download/v${versionString}/${binaryName}`;

  mkdirSync(destDir, { recursive: true });

  console.log(`⬇️  Downloading ${binaryName} from ${url}`);

  const req = https.get(url, (res: any) => {
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

    const file = createWriteStream(dest, { mode: 0o755 });
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
    } catch (e) {}
  });

  req.on("error", (err: any) => {
    // Network or other errors should not fail CI; log and continue
    console.warn(`⚠️  Error downloading binary: ${err.message}. Skipping binary download.`);
  });
} else {
  console.warn(`⚠️  Unsupported platform: ${platform}. Skipping binary download.`);
}
