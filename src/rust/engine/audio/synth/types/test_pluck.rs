use super::*;

#[test]
fn test_pluck_params() {
    let pluck = PluckSynth;
    let mut params = SynthParams::default();

    pluck.modify_params(&mut params);

    assert!(params.attack < 0.01); // Very short attack
    assert_eq!(params.sustain, 0.0); // No sustain
}
