use super::*;
use crate::language::syntax::ast::StatementKind;

#[test]
fn test_print_cases() {
    let mut interp = AudioInterpreter::new(44100);

    // Avoid emitting prints to stdout during the test run
    interp.suppress_print = true;

    // string
    handler::execute_print(&mut interp, &Value::String("hello".to_string())).unwrap();
    // verify log entry recorded
    match interp.events.logs.last().unwrap() {
        (time, message) => {
            assert_eq!(message, "hello");
            assert_eq!(*time, interp.cursor_time);
        }
    }

    // number
    handler::execute_print(&mut interp, &Value::Number(3.14)).unwrap();
    match interp.events.logs.last().unwrap() {
        (_time, message) => {
            assert_eq!(message, "3.14");
        }
    }

    // identifier resolved after let
    handler::handle_let(&mut interp, "x", &Value::Number(7.0)).unwrap();
    handler::execute_print(&mut interp, &Value::Identifier("x".to_string())).unwrap();
    match interp.events.logs.last().unwrap() {
        (_time, message) => {
            assert_eq!(message, "7");
        }
    }

    // array concatenation
    handler::execute_print(
        &mut interp,
        &Value::Array(vec![
            Value::String("a".to_string()),
            Value::Identifier("x".to_string()),
        ]),
    )
    .unwrap();
    match interp.events.logs.last().unwrap() {
        (_time, message) => {
            assert_eq!(message, "a7");
        }
    }

    // map
    let mut m = std::collections::HashMap::new();
    m.insert("k".to_string(), Value::Number(2.0));
    handler::execute_print(&mut interp, &Value::Map(m)).unwrap();
    match interp.events.logs.last().unwrap() {
        (_time, message) => {
            // map debug format should be present
            assert!(message.contains("k"));
        }
    }

    // function call expression
    // create a simple function that returns "pong"
    let ret_stmt = Statement::new(
        StatementKind::Return {
            value: Some(Box::new(Value::String("pong".to_string()))),
        },
        Value::Null,
        0,
        1,
        1,
    );
    let func = Statement::new(
        StatementKind::Function {
            name: "ping".to_string(),
            parameters: Vec::new(),
            body: vec![ret_stmt.clone()],
        },
        Value::Identifier("ping".to_string()),
        0,
        1,
        1,
    );
    interp
        .variables
        .insert("ping".to_string(), Value::Statement(Box::new(func)));

    // Evaluate call expression and print its result
    let call_val = Value::Call {
        name: "ping".to_string(),
        args: Vec::new(),
    };
    handler::execute_print(&mut interp, &call_val).unwrap();
    match interp.events.logs.last().unwrap() {
        (_time, message) => {
            assert_eq!(message, "pong");
        }
    }
}
