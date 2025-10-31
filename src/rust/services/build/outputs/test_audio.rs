#![cfg(test)]
use super::*;
use crate::language::syntax::ast::Value;

#[test]
fn test_parse_fraction_valid() {
    assert_eq!(parse_fraction("3/4"), Some(0.75));
}

#[test]
fn test_parse_fraction_zero_denominator() {
    assert_eq!(parse_fraction("1/0"), None);
}

#[test]
fn test_beats_to_seconds() {
    let seconds = beats_to_seconds(4.0, 120.0);
    assert!((seconds - 2.0).abs() < f32::EPSILON);
}

#[test]
fn test_calculate_rms_empty() {
    let buf: Vec<f32> = vec![];
    assert_eq!(calculate_rms(&buf), 0.0);
}

#[test]
fn test_calculate_rms_nonempty() {
    let buf = vec![1.0f32, -1.0f32];
    let rms = calculate_rms(&buf);
    assert!((rms - 1.0).abs() < 1e-6);
}

#[test]
fn test_normalize_reduces_peak() {
    let mut buf = vec![2.0f32, -2.0f32];
    normalize(&mut buf);
    // peak should be 1.0 after normalization
    assert!(buf.iter().all(|v| v.abs() <= 1.0 + 1e-6));
    assert!((buf[0] - 1.0).abs() < 1e-6);
    assert!((buf[1] + 1.0).abs() < 1e-6);
}

#[test]
fn test_resample_buffer_identity() {
    let src = vec![0.0f32, 1.0f32, 2.0f32, 3.0f32];
    let out = resample_buffer(&src, 1.0, 1);
    assert_eq!(out, src);
}

#[test]
fn test_resample_buffer_downsample() {
    let src = vec![0.0f32, 1.0f32, 2.0f32, 3.0f32];
    let out = resample_buffer(&src, 2.0, 1);
    // with rate=2.0 and 4 frames, expect ~2 frames in result
    assert_eq!(out.len(), 2);
}

#[test]
fn test_apply_delay_zero_does_nothing() {
    let mut buf = vec![0.5f32, 0.0f32];
    let res = apply_delay(&mut buf, 0.0, 44100, 1, 0.5);
    assert!(res.is_ok());
    // delay 0 should keep original samples at start
    assert_eq!(buf[0], 0.5f32);
}

#[test]
fn test_apply_trigger_effects_reverse_and_velocity() {
    let mut buf = vec![0.1f32, 0.2f32, 0.3f32];
    let mut map = std::collections::HashMap::new();
    map.insert("reverse".to_string(), Value::Boolean(true));
    map.insert("velocity".to_string(), Value::Number(64.0));
    let effects = Value::Map(map);

    apply_trigger_effects(&mut buf, &effects, 44100, 1).unwrap();

    // After reverse, original would be [0.3, 0.2, 0.1]
    let expected_first = 0.3f32 * (64.0 / 127.0);
    assert!((buf[0] - expected_first).abs() < 1e-4);
}

// Additional critical helper tests

#[test]
fn test_split_trigger_entity_simple_and_dotted() {
    let (a, b) = split_trigger_entity("kick");
    assert_eq!(a, "kick");
    assert!(b.is_none());

    let (t, maybe) = split_trigger_entity("drums.kick");
    assert_eq!(t, "drums");
    assert_eq!(maybe, Some("kick"));
}

#[test]
fn test_is_master_label_and_register_route_for_alias() {
    // MASTER_INSERT should be considered master
    assert!(is_master_label(MASTER_INSERT));
    assert!(is_master_label(&format!("${}", MASTER_INSERT)));

    let mut inserts = std::collections::HashMap::new();
    inserts.insert(MASTER_INSERT.to_string(), None);
    let mut alias_map = std::collections::HashMap::new();

    let insert_name = register_route_for_alias(&mut inserts, &mut alias_map, "foo", "bus", None);
    // alias map must contain the alias mapped to the returned insert
    assert_eq!(alias_map.get("foo").map(|s| s.as_str()), Some(insert_name.as_str()));
    // inserts must have an entry for the returned insert
    assert!(inserts.contains_key(&insert_name));
    // should not be master insert
    assert_ne!(insert_name, MASTER_INSERT);
}

#[test]
fn test_resolve_sample_reference_absolute_and_relative() {
    let abs = resolve_sample_reference(Path::new("/base"), "/tmp/foo.wav");
    assert_eq!(abs, Path::new("/tmp/foo.wav"));

    let rel = resolve_sample_reference(Path::new("/base"), "samples/bar.wav");
    assert_eq!(rel, Path::new("/base").join("samples/bar.wav"));
}

#[test]
fn test_default_bank_alias_sanitization() {
    let alias = default_bank_alias("path/to/my-bank");
    assert_eq!(alias, "my_bank");

    let alias2 = default_bank_alias("simplebank");
    assert_eq!(alias2, "simplebank");
}

#[test]
fn test_value_as_string_variants() {
    assert_eq!(value_as_string(&Value::String("abc".to_string())), Some("abc".to_string()));
    assert_eq!(value_as_string(&Value::Identifier("id".to_string())), Some("id".to_string()));
    assert_eq!(value_as_string(&Value::Number(3.14)), Some("3.14".to_string()));
}

#[test]
fn test_duration_in_seconds_variants() {
    // Milliseconds
    let d = DurationValue::Milliseconds(500.0);
    assert_eq!(duration_in_seconds(&d, 120.0), Some(0.5));

    // Beats (fraction)
    let d2 = DurationValue::Beat("1/2".to_string());
    let seconds = duration_in_seconds(&d2, 120.0).unwrap();
    assert!((seconds - beats_to_seconds(0.5, 120.0)).abs() < 1e-6);
}
