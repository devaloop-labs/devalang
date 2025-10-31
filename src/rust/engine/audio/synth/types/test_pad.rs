use super::*;

#[test]
fn test_pad_params() {
    let pad = PadSynth;
    let mut params = SynthParams::default();

    pad.modify_params(&mut params);

    assert!(params.attack > 0.2); // Slow attack
    assert!(params.sustain > 0.8); // High sustain
    assert!(params.release > 0.5); // Long release
}
