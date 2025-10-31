use super::*;

#[test]
fn test_parse_simple_effect() {
    let result = parse_single_effect("reverb").unwrap();
    assert_eq!(result.0, "reverb");
    assert!(matches!(result.1, Value::Null));
}

#[test]
fn test_parse_boolean_effect() {
    let result = parse_single_effect("reverse(false)").unwrap();
    assert_eq!(result.0, "reverse");
    assert!(matches!(result.1, Value::Boolean(false)));
}

#[test]
fn test_parse_numeric_effect() {
    let result = parse_single_effect("speed(1.2)").unwrap();
    assert_eq!(result.0, "speed");
    assert!(matches!(result.1, Value::Number(n) if (n - 1.2).abs() < f32::EPSILON));
}

#[test]
fn test_parse_map_effect() {
    let result = parse_single_effect("drive({amount: 0.7, mix: 0.5})").unwrap();
    assert_eq!(result.0, "drive");
    if let Value::Map(map) = result.1 {
        assert!(
            matches!(map.get("amount"), Some(Value::Number(n)) if (*n - 0.7).abs() < f32::EPSILON)
        );
        assert!(
            matches!(map.get("mix"), Some(Value::Number(n)) if (*n - 0.5).abs() < f32::EPSILON)
        );
    } else {
        panic!("Expected map parameters");
    }
}

#[test]
fn test_parse_effect_chain() {
    let chain = "reverse(false) -> speed(1.2) -> drive({amount: 0.7, mix: 0.5})";
    let result = parse_chained_effects(chain).unwrap();
    if let Value::Map(effects) = result {
        assert!(effects.contains_key("reverse"));
        assert!(effects.contains_key("speed"));
        assert!(effects.contains_key("drive"));
    } else {
        panic!("Expected map of effects");
    }
}
