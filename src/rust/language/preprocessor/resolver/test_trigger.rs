use super::*;

#[test]
fn test_resolve_trigger_with_duration_identifier() {
    let mut vars = HashMap::new();
    vars.insert("short".to_string(), Value::Number(100.0));

    let stmt = Statement::new(
        StatementKind::Trigger {
            entity: "kick".to_string(),
            duration: DurationValue::Identifier("short".to_string()),
            effects: None,
        },
        Value::Null,
        0,
        1,
        1,
    );

    let resolved = resolve_trigger(
        &stmt,
        "kick",
        &DurationValue::Identifier("short".to_string()),
        &None,
        &vars,
    );

    if let StatementKind::Trigger { duration, .. } = &resolved.kind {
        assert!(
            matches!(duration, DurationValue::Milliseconds(ms) if (ms - 100.0).abs() < f32::EPSILON)
        );
    } else {
        panic!("Expected trigger statement");
    }
}
