import { createWriteStream, mkdirSync } from "fs";
import { join } from "path";
import { https } from "follow-redirects";
import fs from "fs";
import path from "path";

const projectVersionPath = path.join(
  __dirname,
  "../../project-version.json"
);

const version = fs.readFileSync(projectVersionPath, "utf-8").trim();
const versionString = JSON.parse(version).version;

const platform = process.platform;

let binaryName: string;
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

const destDir = join(__dirname, "..", "..", "out-tsc", "bin");
const dest = join(destDir, binaryName);

const url = `https://github.com/devaloop-labs/devalang/releases/download/v${versionString}/${binaryName}`;

mkdirSync(destDir, { recursive: true });

console.log(`⬇️  Downloading ${binaryName} from ${url}`);

https.get(url, (res: any) => {
  if (res.statusCode !== 200) {
    console.error(`❌ Failed (HTTP ${res.statusCode})`);
    process.exit(1);
  }
  const file = createWriteStream(dest, { mode: 0o755 });
  res.pipe(file);
  file.on("finish", () => {
    file.close();
    console.log(`✅ Downloaded ${binaryName} to ${dest}`);
  });
}).on("error", (err) => {
  console.error(`❌ Error: ${err.message}`);
  process.exit(1);
});
