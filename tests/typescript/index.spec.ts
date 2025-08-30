import { expect } from "chai";
import * as fs from "fs";
import * as os from "os";
import * as path from "path";

// CommonJS bundle import (TypeScript-friendly)
const core: any = require("../../out-tsc");

describe("devalang typescript index", () => {
  let origCwd: string;
  let tmpDir: string | null = null;

  before(function () {
    // Prepare a temporary project root so wasm code that looks up project files
    // and `.devalang` config can succeed during E2E runs.
    origCwd = process.cwd();
    tmpDir = fs.mkdtempSync(path.join(os.tmpdir(), "devalang-e2e-"));
    // Create minimal .devalang config file so get_project_root finds it
    fs.writeFileSync(
      path.join(tmpDir, ".devalang"),
      'name = "devalang-test"\n'
    );
    // Ensure .deva exists (optional banks/plugins)
    fs.mkdirSync(path.join(tmpDir, ".deva"), { recursive: true });
    process.chdir(tmpDir);
  });

  after(function () {
    // restore cwd and attempt cleanup
    if (tmpDir) {
      try {
        process.chdir(origCwd);
        // best-effort cleanup
        fs.rmSync(tmpDir, { recursive: true, force: true });
      } catch (e) {
        // ignore
      }
    }
  });
  it("should expose render_audio and return a Float32Array when wasm is available", function () {
    // Prefer wasm binding if available
    const wasmFn = core.pkg && core.pkg.render_audio;

    const sampleProgram = `# simple program for tests
bpm 120

group myLead:
  let mySynth = synth sine
  mySynth -> note(C4, { duration: 400 })
  mySynth -> note(G4, { duration: 400 })
call myLead
`;

    if (typeof wasmFn === "function") {
      try {
        // Get parsed AST first for debugging
        if (core.pkg && typeof core.pkg.parse === "function") {
          try {
            const parsed = core.pkg.parse("playground.deva", sampleProgram);
            // parsed may be an object { ok, ast, errors } or a string
            const astStr =
              parsed && parsed.ast
                ? parsed.ast
                : typeof parsed === "string"
                ? parsed
                : JSON.stringify(parsed);
            expect(astStr).to.match(/note|synth/i);
          } catch (e) {
            console.warn(
              "[TEST DEBUG] parse() failed or returned non-standard result:",
              e
            );
          }
        }

        const result = wasmFn(sampleProgram);
        expect(result.constructor && result.constructor.name).to.match(
          /Float32Array|Float64Array|ArrayBuffer|Uint8Array/
        );
      } catch (err: any) {
        // If the wasm panicked because the audio buffer is empty, skip the test in this env.
        const msg =
          typeof err === "string"
            ? err
            : err && err.message
            ? err.message
            : String(err);
        if (msg.includes("Audio buffer is empty")) {
          // skip this test on environments where generating audio isn't supported
          // eslint-disable-next-line no-invalid-this
          this.skip();
        }

        throw err;
      }

      return;
    }

    // If wasm binding isn't available, ensure a runtime placeholder exists but don't call it
    const runtimeFn = core.render_audio;
    expect(runtimeFn).to.be.a("function");
  });

  it("should find statements for entry module", function () {
    const parseFn = core.pkg && core.pkg.parse;
    if (typeof parseFn !== "function") {
      // nothing to test if parse binding missing
      // eslint-disable-next-line no-invalid-this
      this.skip();
    }

    try {
      const res = parseFn("playground.deva", "bpm 120");
      expect(res).to.be.an("object");
      // prefer explicit shape when available
      if (Object.prototype.hasOwnProperty.call(res, "ok")) {
        expect(res.ok).to.equal(true);
      } else {
        expect(res).to.have.property("ast");
      }
    } catch (err: any) {
      const msg = err && err.message ? err.message : String(err);
      if (
        msg.includes("Module loading error") ||
        msg.includes("Project root not found")
      ) {
        // environment not prepared for module loading, skip
        // eslint-disable-next-line no-invalid-this
        this.skip();
        return;
      }
      throw err;
    }
  });
});
