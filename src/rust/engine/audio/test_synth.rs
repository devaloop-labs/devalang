use super::*;

#[test]
fn test_oscillator_sine() {
    let sample = oscillator_sample("sine", 440.0, 0.0);
    assert!((sample - 0.0).abs() < 0.01);
}

#[test]
fn test_oscillator_square() {
    let sample = oscillator_sample("square", 440.0, 0.0);
    assert!((sample - 1.0).abs() < 0.01);
}

#[test]
fn test_adsr_attack() {
    let envelope = adsr_envelope(500, 1000, 500, 1000, 500, 0.7);
    assert!(envelope > 0.4 && envelope < 0.6); // Mid-attack
}

#[test]
fn test_midi_to_frequency() {
    let a4 = midi_to_frequency(69);
    assert!((a4 - 440.0).abs() < 0.1);

    let c4 = midi_to_frequency(60);
    assert!((c4 - 261.63).abs() < 0.5);
}
