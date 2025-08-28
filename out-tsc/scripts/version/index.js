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
const path_1 = __importDefault(require("path"));
const bump_1 = require("./bump");
const sync_1 = require("./sync");
const fetch_1 = require("./fetch");
const bumpType = process.argv[2] || "patch";
(() => __awaiter(void 0, void 0, void 0, function* () {
    const projectVersionPath = path_1.default.join(__dirname, "../../../project-version.json");
    try {
        const newVersion = yield (0, bump_1.bumpVersion)(bumpType, projectVersionPath);
        yield (0, fetch_1.fetchVersion)(projectVersionPath);
        yield (0, sync_1.syncVersion)(projectVersionPath);
        console.log(`✅ Project version updated to : ${newVersion}`);
    }
    catch (error) {
        console.error(`❌ Error updating project version: ${error}`);
        process.exit(1);
    }
}))();
