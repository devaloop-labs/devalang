#!/usr/bin/env node
"use strict";
var __importDefault = (this && this.__importDefault) || function (mod) {
    return (mod && mod.__esModule) ? mod : { "default": mod };
};
Object.defineProperty(exports, "__esModule", { value: true });
const child_process_1 = require("child_process");
const path_1 = __importDefault(require("path"));
const binaryPath = path_1.default.join(__dirname, "devalang.exe");
const subCommand = process.argv[2] || "help";
const args = process.argv.slice(2);
const child = (0, child_process_1.spawn)(binaryPath, args, { stdio: "inherit" });
child.on("exit", (code) => process.exit(code));
