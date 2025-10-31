use super::*;
use crate::language::syntax::ast::StatementKind;

#[test]
fn test_return_inside_function_propagates_to_caller() {
    let mut interp = AudioInterpreter::new(44100);

    // Build function body: return "ok"
    let ret_stmt = Statement::new(
        StatementKind::Return {
            value: Some(Box::new(Value::String("ok".to_string()))),
        },
        Value::Null,
        0,
        1,
        1,
    );
    let body = vec![ret_stmt];

    // Create function statement and store as variable
    let func_stmt = Statement::new(
        StatementKind::Function {
            name: "f".to_string(),
            parameters: Vec::new(),
            body: body.clone(),
        },
        Value::Identifier("f".to_string()),
        0,
        1,
        1,
    );

    interp
        .variables
        .insert("f".to_string(), Value::Statement(Box::new(func_stmt)));

    // Call the function via the new call_function API and assert returned value
    let res = handler::call_function(&mut interp, "f", &[]).expect("call succeeded");
    match res {
        Value::String(s) => assert_eq!(s, "ok"),
        other => panic!("expected String return, got {:?}", other),
    }
}

#[test]
fn test_return_outside_function_errors() {
    let mut interp = AudioInterpreter::new(44100);

    // Create a bare return statement at top level
    let ret_stmt = Statement::new(
        StatementKind::Return {
            value: Some(Box::new(Value::Number(1.0))),
        },
        Value::Null,
        0,
        1,
        1,
    );

    let res = collector::collect_events(&mut interp, &vec![ret_stmt]);
    assert!(res.is_err());
}
