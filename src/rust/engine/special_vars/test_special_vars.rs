use super::*;

#[test]
fn test_is_special_var() {
    assert!(is_special_var("$time"));
    assert!(is_special_var("$beat"));
    assert!(is_special_var("$random"));
    assert!(!is_special_var("time"));
    assert!(!is_special_var("myVar"));
}

#[test]
fn test_resolve_time_vars() {
    let mut context = SpecialVarContext::default();
    context.update_time(2.0);

    let time = resolve_special_var("$time", &context);
    assert_eq!(time, Some(Value::Number(2.0)));

    let beat = resolve_special_var("$beat", &context);
    assert!(matches!(beat, Some(Value::Number(_))));
}

#[test]
fn test_resolve_random_vars() {
    let context = SpecialVarContext::default();

    let rand1 = resolve_special_var("$random", &context);
    assert!(matches!(rand1, Some(Value::Number(_))));

    let rand2 = resolve_special_var("$random.noise", &context);
    assert!(matches!(rand2, Some(Value::Number(_))));
}

#[test]
fn test_context_update_time() {
    let mut context = SpecialVarContext::new(120.0, 44100);
    context.update_time(1.0);

    assert_eq!(context.current_time, 1.0);
    assert!(context.current_beat > 0.0);
}

#[test]
fn test_parse_random_range() {
    let result = parse_random_range("$random.range(0, 10)");
    assert!(result.is_some());

    if let Some(Value::Number(n)) = result {
        assert!(n >= 0.0 && n <= 10.0);
    }
}
