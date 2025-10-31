use super::*;

#[test]
fn test_effect_registry() {
    let registry = EffectRegistry::new();

    // Test synth context
    assert!(registry.is_effect_available("reverb", true));
    assert!(!registry.is_effect_available("speed", true));

    // Test trigger context
    assert!(registry.is_effect_available("speed", false));
    assert!(registry.is_effect_available("reverb", false));

    // Test getting processors
    assert!(registry.get_effect("reverb", true).is_some());
    assert!(registry.get_effect("speed", true).is_none());
    assert!(registry.get_effect("speed", false).is_some());
}

#[test]
fn test_reverse_processor() {
    let mut processor = ReverseProcessor::new(true);
    let mut samples = vec![1.0, 2.0, 3.0, 4.0];
    processor.process(&mut samples, 44100);
    assert_eq!(samples, vec![4.0, 3.0, 2.0, 1.0]);
}

#[test]
fn test_speed_processor() {
    let mut processor = SpeedProcessor::new(2.0); // Double speed
    let mut samples = vec![1.0, 2.0, 3.0, 4.0];
    processor.process(&mut samples, 44100);
    // Since we can't change the slice length in-place, the processed values are written
    // into the beginning of the slice and the remainder zeroed.
    assert_eq!(samples[0], 1.0);
    assert_eq!(samples[1], 3.0);
    assert_eq!(samples[2], 0.0);
    assert_eq!(samples[3], 0.0);
}
