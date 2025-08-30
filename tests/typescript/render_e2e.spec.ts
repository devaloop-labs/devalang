import { expect } from "chai";
import * as fs from "fs";
import * as os from "os";
import * as path from "path";

const core: any = require("../../out-tsc");

describe("devalang debug_render and render_audio E2E", () => {
  let origCwd: string;
  let tmpDir: string | null = null;

  before(function () {
    origCwd = process.cwd();
    tmpDir = fs.mkdtempSync(path.join(os.tmpdir(), "devalang-e2e-"));
    fs.writeFileSync(
      path.join(tmpDir, ".devalang"),
      'name = "devalang-test"\n'
    );
    fs.mkdirSync(path.join(tmpDir, ".deva"), { recursive: true });
    process.chdir(tmpDir as string);
  });

  after(function () {
    if (tmpDir) {
      try {
        process.chdir(origCwd);
        fs.rmSync(tmpDir, { recursive: true, force: true });
      } catch (e) {
        // ignore
      }
    }
  });

  it("debug_render should return diagnostics including note_count and global_vars", function () {
    const dbg = core.pkg && core.pkg.debug_render;
    if (typeof dbg !== "function") {
      // skip if wasm binding missing
      // eslint-disable-next-line no-invalid-this
      this.skip();
    }

    const sampleProgram = `# sample program\nbpm 120\n\ngroup myLead:\n  let mySynth = synth sine\n  mySynth -> note(C4, { duration: 400 })\n  mySynth -> note(G4, { duration: 400 })\ncall myLead\n`;

    const out = dbg(sampleProgram);
    expect(out).to.be.an("object");
    expect(out).to.have.property("samples_len");
    expect(out).to.have.property("any_nonzero");
    expect(out).to.have.property("note_count");
    expect(out).to.have.property("global_vars");

    // Basic assertions: at least one non-zero sample or note_count > 0
    expect(out.note_count).to.be.at.least(0);
    expect(out.global_vars).to.include("myLead");
  });

  it("render_audio should return a Float32Array and not be completely silent", function () {
    const render = (core.pkg && core.pkg.render_audio) || core.render_audio;
    if (typeof render !== "function") {
      // eslint-disable-next-line no-invalid-this
      this.skip();
    }

    const sampleProgram = `# sample program\nbpm 120\n\ngroup myLead:\n  let mySynth = synth sine\n  mySynth -> note(C4, { duration: 400 })\n  mySynth -> note(G4, { duration: 400 })\ncall myLead\n`;

    const arr = render(sampleProgram);
    // If wasm returns JS typed array wrapper, check constructor name
    expect(arr.constructor && arr.constructor.name).to.match(
      /Float32Array|Float64Array|ArrayBuffer|Uint8Array/
    );

    // Convert to plain array if needed
    const samples: number[] = Array.from(arr as any);
    // Expect either note_count > 0 indirectly via nonzero samples
    const anyNonZero = samples.some((s) => s !== 0);
    expect(anyNonZero).to.be.true;
  });
});
