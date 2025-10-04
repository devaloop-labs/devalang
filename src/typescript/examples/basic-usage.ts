/**
 * Example: Using Devalang TypeScript API
 * 
 * This example demonstrates how to use the Devalang API
 * to parse, render, and export audio.
 */

import * as devalang from '../index';

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
  } else {
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
