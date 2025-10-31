use super::*;

#[test]
fn test_lfo_sine() {
    let params = LfoParams {
        rate: LfoRate::Hz(1.0),
        depth: 1.0,
        waveform: LfoWaveform::Sine,
        target: LfoTarget::Volume,
        phase: 0.0,
    };

    // At t=0, phase=0, sin(0)=0
    let v0 = generate_lfo_value(&params, 0.0, 120.0);
    assert!((v0 - 0.0).abs() < 0.001);

    // At t=0.25, phase=0.25, sin(π/2)=1
    let v1 = generate_lfo_value(&params, 0.25, 120.0);
    assert!((v1 - 1.0).abs() < 0.001);

    // At t=0.5, phase=0.5, sin(π)=0
    let v2 = generate_lfo_value(&params, 0.5, 120.0);
    assert!((v2 - 0.0).abs() < 0.001);
}

#[test]
fn test_lfo_triangle() {
    let params = LfoParams {
        rate: LfoRate::Hz(1.0),
        depth: 1.0,
        waveform: LfoWaveform::Triangle,
        target: LfoTarget::Pitch,
        phase: 0.0,
    };

    let v0 = generate_lfo_value(&params, 0.0, 120.0);
    assert!((v0 - 1.0).abs() < 0.001);

    let v1 = generate_lfo_value(&params, 0.25, 120.0);
    assert!((v1 - 0.0).abs() < 0.001);

    let v2 = generate_lfo_value(&params, 0.5, 120.0);
    assert!((v2 - (-1.0)).abs() < 0.001);
}

#[test]
fn test_apply_modulation() {
    let params = LfoParams {
        rate: LfoRate::Hz(1.0),
        depth: 0.5, // 50% depth
        waveform: LfoWaveform::Sine,
        target: LfoTarget::Volume,
        phase: 0.0,
    };

    // Center at 0.5, range of 0.2
    // At t=0.25 (peak), should be 0.5 + (1.0 * 0.5 * 0.2) = 0.6
    let modulated = apply_lfo_modulation(&params, 0.25, 120.0, 0.5, 0.2);
    assert!((modulated - 0.6).abs() < 0.001);
}

#[test]
fn test_tempo_sync_rate() {
    // 1/4 beat at 120 BPM should be 8 Hz (120/60 * 4)
    let rate = LfoRate::from_value("1/4");
    assert_eq!(rate.to_hz(120.0), 8.0);

    // 1/8 beat at 120 BPM should be 16 Hz
    let rate2 = LfoRate::from_value("1/8");
    assert_eq!(rate2.to_hz(120.0), 16.0);

    // Regular Hz should stay the same
    let rate3 = LfoRate::from_value("4.0");
    assert_eq!(rate3.to_hz(120.0), 4.0);
}
