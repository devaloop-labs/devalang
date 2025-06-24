import fs from "fs";
import path from "path";

const source = path.join(__dirname, "..", "..", "target", "release", "devalang.exe");
const destination = path.join(__dirname, "..", "bin", "devalang.exe");

fs.copyFileSync(source, destination);
fs.chmodSync(destination, 0o755); 
