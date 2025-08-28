#!/usr/bin/env node

import { spawn } from "child_process";
import * as path from "path";

let binaryName: string;

switch (process.platform) {
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
    console.error(`Unsupported platform: ${process.platform}`);
    process.exit(1);
}

const binaryPath = path.join(__dirname, "bin", binaryName);

const args = process.argv.slice(2);
const child = spawn(binaryPath, args, { stdio: "inherit" });

child.on("exit", (code) => process.exit(code ?? 1));
