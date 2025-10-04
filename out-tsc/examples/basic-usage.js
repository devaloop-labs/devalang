"use strict";
/**
 * Example: Using Devalang TypeScript API
 *
 * This example demonstrates how to use the Devalang API
 * to parse, render, and export audio.
 */
var __createBinding = (this && this.__createBinding) || (Object.create ? (function(o, m, k, k2) {
    if (k2 === undefined) k2 = k;
    var desc = Object.getOwnPropertyDescriptor(m, k);
    if (!desc || ("get" in desc ? !m.__esModule : desc.writable || desc.configurable)) {
      desc = { enumerable: true, get: function() { return m[k]; } };
    }
    Object.defineProperty(o, k2, desc);
}) : (function(o, m, k, k2) {
    if (k2 === undefined) k2 = k;
    o[k2] = m[k];
}));
var __setModuleDefault = (this && this.__setModuleDefault) || (Object.create ? (function(o, v) {
    Object.defineProperty(o, "default", { enumerable: true, value: v });
}) : function(o, v) {
    o["default"] = v;
});
var __importStar = (this && this.__importStar) || (function () {
    var ownKeys = function(o) {
        ownKeys = Object.getOwnPropertyNames || function (o) {
            var ar = [];
            for (var k in o) if (Object.prototype.hasOwnProperty.call(o, k)) ar[ar.length] = k;
            return ar;
        };
        return ownKeys(o);
    };
    return function (mod) {
        if (mod && mod.__esModule) return mod;
        var result = {};
        if (mod != null) for (var k = ownKeys(mod), i = 0; i < k.length; i++) if (k[i] !== "default") __createBinding(result, mod, k[i]);
        __setModuleDefault(result, mod);
        return result;
    };
})();
Object.defineProperty(exports, "__esModule", { value: true });
const devalang = __importStar(require("../index"));
async function main() {
    console.log('🎵 Devalang TypeScript API Example\n');
    // 1. Parse Devalang code
    console.log('1️⃣ Parsing code...');
    const parseResult = await devalang.parse('example.deva', `
    bpm 120
    let s = synth sine {}
    s -> note(C4, { duration: 500 })
  `);
    if (parseResult.success) {
        console.log(`✅ Parsed ${parseResult.statements.length} statements\n`);
    }
    else {
        console.error('❌ Parse errors:', parseResult.errors);
        return;
    }
    // 2. Render audio
    console.log('2️⃣ Rendering audio...');
    const audioCode = `
    bpm 120
    let mySynth = synth sine {
      attack: 0.01,
      release: 0.3
    }
    mySynth -> note(A4, { duration: 1000, velocity: 80 })
  `;
    const audioBuffer = await devalang.renderAudio(audioCode, {
        sampleRate: 44100,
        bpm: 120
    });
    console.log(`✅ Rendered ${audioBuffer.length} samples (${(audioBuffer.length / 44100).toFixed(2)}s)\n`);
    // 3. Debug render with metadata
    console.log('3️⃣ Debug render...');
    const debugResult = await devalang.debugRender(audioCode);
    console.log(`📊 Metadata:`);
    console.log(`   - Duration: ${debugResult.duration.toFixed(2)}s`);
    console.log(`   - Sample Rate: ${debugResult.sampleRate} Hz`);
    console.log(`   - Events: ${debugResult.eventCount}`);
    console.log(`   - BPM: ${debugResult.bpm}\n`);
    // 4. Export MIDI
    console.log('4️⃣ Exporting MIDI...');
    const midiCode = `
    bpm 140
    let piano = synth sine {}
    piano -> note(C4, { duration: 500 })
    piano -> note(E4, { duration: 500 })
    piano -> note(G4, { duration: 1000 })
  `;
    const midiBytes = await devalang.renderMidi(midiCode, {
        bpm: 140
    });
    console.log(`✅ Exported MIDI: ${midiBytes.length} bytes\n`);
    // 5. Export WAV
    console.log('5️⃣ Exporting WAV...');
    const wavBytes = await devalang.renderWavPreview(audioCode);
    console.log(`✅ Exported WAV: ${wavBytes.length} bytes\n`);
    // 6. Get metadata without rendering
    console.log('6️⃣ Getting code metadata...');
    const metadata = await devalang.getCodeMetadata(audioCode);
    console.log(`📝 Code info:`);
    console.log(`   - Statements: ${metadata.statementCount}`);
    console.log(`   - BPM: ${metadata.bpm}`);
    console.log(`   - Sample Rate: ${metadata.sampleRate} Hz\n`);
    console.log('✨ Example completed successfully!');
}
// Run example
main().catch(error => {
    console.error('❌ Error:', error);
    process.exit(1);
});
//# sourceMappingURL=basic-usage.js.map