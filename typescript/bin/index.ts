#!/usr/bin/env node

import { spawn } from "child_process";
import path from "path";

const binaryPath = path.join(__dirname, "devalang.exe");

const subCommand = process.argv[2] || "help";

const args = process.argv.slice(2);
const child = spawn(binaryPath, args, { stdio: "inherit" });


child.on("exit", (code) => process.exit(code));