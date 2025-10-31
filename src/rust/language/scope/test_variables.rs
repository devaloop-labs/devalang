use super::*;

#[test]
fn test_basic_let() {
    let mut table = VariableTable::new();
    table.set_with_type("x".to_string(), Value::Number(42.0), BindingType::Let);
    assert_eq!(table.get("x"), Some(&Value::Number(42.0)));
}

#[test]
fn test_const_cannot_reassign() {
    let mut table = VariableTable::new();
    table.set_with_type("x".to_string(), Value::Number(42.0), BindingType::Const);

    let result = table.update("x", Value::Number(100.0));
    assert!(result.is_err());
    assert_eq!(table.get("x"), Some(&Value::Number(42.0)));
}

#[test]
fn test_let_can_reassign() {
    let mut table = VariableTable::new();
    table.set_with_type("x".to_string(), Value::Number(42.0), BindingType::Let);

    let result = table.update("x", Value::Number(100.0));
    assert!(result.is_ok());
    assert_eq!(table.get("x"), Some(&Value::Number(100.0)));
}

#[test]
fn test_scoped_access() {
    let mut parent = VariableTable::new();
    parent.set("global".to_string(), Value::Number(1.0));

    let mut child = VariableTable::with_parent(parent);
    child.set("local".to_string(), Value::Number(2.0));

    assert_eq!(child.get("local"), Some(&Value::Number(2.0)));
    assert_eq!(child.get("global"), Some(&Value::Number(1.0)));
}

#[test]
fn test_var_hoisting() {
    let mut parent = VariableTable::new();
    parent.set_with_type("x".to_string(), Value::Number(1.0), BindingType::Var);

    let mut child = VariableTable::with_parent(parent);
    child.set_with_type("x".to_string(), Value::Number(2.0), BindingType::Var);

    // Verify child has access to the updated var
    assert_eq!(child.get("x"), Some(&Value::Number(2.0)));

    // Extract parent and verify it was updated
    if let Some(parent_box) = child.parent {
        assert_eq!(parent_box.get("x"), Some(&Value::Number(2.0)));
    } else {
        panic!("Test error: Parent should exist in this test scenario");
    }
}

#[test]
fn test_shadowing() {
    let mut parent = VariableTable::new();
    parent.set("x".to_string(), Value::Number(1.0));

    let mut child = VariableTable::with_parent(parent.clone());
    child.set("x".to_string(), Value::Number(2.0));

    assert_eq!(child.get("x"), Some(&Value::Number(2.0)));
    assert_eq!(parent.get("x"), Some(&Value::Number(1.0)));
}
