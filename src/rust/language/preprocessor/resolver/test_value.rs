use super::*;

#[test]
fn test_resolve_identifier() {
    let mut vars = HashMap::new();
    vars.insert("x".to_string(), Value::Number(42.0));

    let result = resolve_value(&Value::Identifier("x".to_string()), &vars, 0);
    assert!(matches!(result, Value::Number(n) if (n - 42.0).abs() < f32::EPSILON));
}

#[test]
fn test_resolve_chained_identifier() {
    let mut vars = HashMap::new();
    vars.insert("x".to_string(), Value::Identifier("y".to_string()));
    vars.insert("y".to_string(), Value::Number(100.0));

    let result = resolve_value(&Value::Identifier("x".to_string()), &vars, 0);
    assert!(matches!(result, Value::Number(n) if (n - 100.0).abs() < f32::EPSILON));
}

#[test]
fn test_resolve_map() {
    let mut vars = HashMap::new();
    vars.insert("gain".to_string(), Value::Number(0.8));

    let mut input_map = HashMap::new();
    input_map.insert("vol".to_string(), Value::Identifier("gain".to_string()));

    let result = resolve_value(&Value::Map(input_map), &vars, 0);
    if let Value::Map(m) = result {
        if let Some(Value::Number(n)) = m.get("vol") {
            assert!((n - 0.8).abs() < f32::EPSILON);
            return;
        }
    }
    panic!("Expected resolved map with vol=0.8");
}
