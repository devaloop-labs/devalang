"use strict";
var __awaiter = (this && this.__awaiter) || function (thisArg, _arguments, P, generator) {
    function adopt(value) { return value instanceof P ? value : new P(function (resolve) { resolve(value); }); }
    return new (P || (P = Promise))(function (resolve, reject) {
        function fulfilled(value) { try { step(generator.next(value)); } catch (e) { reject(e); } }
        function rejected(value) { try { step(generator["throw"](value)); } catch (e) { reject(e); } }
        function step(result) { result.done ? resolve(result.value) : adopt(result.value).then(fulfilled, rejected); }
        step((generator = generator.apply(thisArg, _arguments || [])).next());
    });
};
var __importDefault = (this && this.__importDefault) || function (mod) {
    return (mod && mod.__esModule) ? mod : { "default": mod };
};
Object.defineProperty(exports, "__esModule", { value: true });
exports.bumpVersion = void 0;
const fs_1 = __importDefault(require("fs"));
const bumpVersion = (bumpType, projectVersionPath) => __awaiter(void 0, void 0, void 0, function* () {
    const versionData = JSON.parse(fs_1.default.readFileSync(projectVersionPath, "utf-8"));
    const versionRegex = /^(\d+)\.(\d+)\.(\d+)(?:-([\w.]+))?$/;
    const match = versionData.version.match(versionRegex);
    if (!match) {
        throw new Error("Invalid version format in project-version.json");
    }
    if (!bumpType) {
        console.error("❌ Please specify a version type (major, minor, patch, pre)");
        process.exit(1);
    }
    let [_, major, minor, patch, pre] = match;
    let nextVersion = "";
    switch (bumpType) {
        case "major":
            nextVersion = `${+major + 1}.0.0`;
            break;
        case "minor":
            nextVersion = `${major}.${+minor + 1}.0`;
            break;
        case "patch":
            nextVersion = `${major}.${minor}.${+patch + 1}`;
            break;
        case "pre":
            const [preid, prenumber] = (pre || "alpha.0").split(".");
            nextVersion = `${major}.${minor}.${patch}-${preid}.${+prenumber + 1}`;
            break;
        default:
            console.error("❌ Version type non-recognized (major, minor, patch, pre)");
            process.exit(1);
    }
    versionData.version = nextVersion;
    fs_1.default.writeFileSync(projectVersionPath, JSON.stringify(versionData, null, 2));
    return nextVersion;
});
exports.bumpVersion = bumpVersion;
