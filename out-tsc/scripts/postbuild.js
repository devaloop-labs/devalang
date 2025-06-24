"use strict";
var __importDefault = (this && this.__importDefault) || function (mod) {
    return (mod && mod.__esModule) ? mod : { "default": mod };
};
Object.defineProperty(exports, "__esModule", { value: true });
const fs_1 = __importDefault(require("fs"));
const path_1 = __importDefault(require("path"));
const source = path_1.default.join(__dirname, "..", "..", "target", "release", "devalang.exe");
const destination = path_1.default.join(__dirname, "..", "bin", "devalang.exe");
fs_1.default.copyFileSync(source, destination);
fs_1.default.chmodSync(destination, 0o755);
