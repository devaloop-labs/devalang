use super::*;

#[test]
fn test_global_store_variables() {
    let store = GlobalStore::new();
    store.set_variable("x".to_string(), Value::Number(42.0));

    let result = store.get_variable("x");
    assert!(matches!(result, Some(Value::Number(n)) if (n - 42.0).abs() < f32::EPSILON));
}

#[test]
fn test_global_store_modules() {
    let store = GlobalStore::new();
    let info = ModuleInfo {
        path: "test".to_string(),
        variables: HashMap::new(),
    };

    store.register_module("test".to_string(), info.clone());
    let result = store.get_module("test");
    assert!(result.is_some());
}
