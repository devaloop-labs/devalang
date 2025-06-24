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
exports.syncVersion = void 0;
const fs_1 = __importDefault(require("fs"));
const path_1 = __importDefault(require("path"));
const syncVersion = (projectVersionPath) => __awaiter(void 0, void 0, void 0, function* () {
    const version = fs_1.default.readFileSync(projectVersionPath, "utf-8").trim();
    const versionString = JSON.parse(version).version;
    // Package.json
    const pkgPath = path_1.default.join(__dirname, "..", "..", "..", "package.json");
    const pkgJson = JSON.parse(fs_1.default.readFileSync(pkgPath, "utf-8"));
    pkgJson.version = versionString;
    fs_1.default.writeFileSync(pkgPath, JSON.stringify(pkgJson, null, 2));
    // Cargo.toml
    const cargoPath = path_1.default.join(__dirname, "..", "..", "..", "Cargo.toml");
    const cargoToml = fs_1.default.readFileSync(cargoPath, "utf-8");
    const updatedCargo = cargoToml.replace(/(version\s*=\s*")[^"]*(")/, `$1${versionString}$2`);
    fs_1.default.writeFileSync(cargoPath, updatedCargo);
});
exports.syncVersion = syncVersion;
