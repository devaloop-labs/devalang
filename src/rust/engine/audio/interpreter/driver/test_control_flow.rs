use anyhow::Result;

use crate::engine::audio::interpreter::driver::AudioInterpreter;
use crate::language::syntax::ast::{Statement, StatementKind, Value};

#[test]
fn test_break_in_loop_exits_early() -> Result<()> {
    let mut interp = AudioInterpreter::new(44100);

    // Create a loop of 10 iterations but body contains a print then a break
    let print_stmt = Statement::print("tick", 1, 1);
    let break_stmt = Statement::new(StatementKind::Break, Value::Null, 0, 2, 1);

    // Sanity: executing a standalone break should set the interpreter.break_flag
    let break_only = Statement::new(StatementKind::Break, Value::Null, 0, 10, 1);
    interp.collect_events(&[break_only])?;
    assert!(
        interp.break_flag,
        "break did not set break_flag when executed standalone"
    );
    // clear it for the real test
    interp.break_flag = false;

    // Sanity: executing the body (print then break) directly should produce one log and set break_flag
    let direct_print = Statement::print("tick", 1, 1);
    let direct_break = Statement::new(StatementKind::Break, Value::Null, 0, 11, 1);
    // clear any previous logs
    interp.events.logs.clear();
    interp.collect_events(&[direct_print.clone(), direct_break.clone()])?;
    assert!(
        interp.break_flag,
        "break in direct body did not set break_flag"
    );
    assert_eq!(
        interp.events.logs.len(),
        1,
        "expected one log from direct body execution"
    );
    // reset for main loop test
    interp.break_flag = false;
    interp.events.logs.clear();

    let loop_stmt = Statement::new(
        StatementKind::Loop {
            count: Value::Number(10.0),
            body: vec![print_stmt, break_stmt],
        },
        Value::Null,
        0,
        1,
        1,
    );

    // Collect events for the loop
    interp.collect_events(&[loop_stmt])?;

    // Only the first print should have been scheduled (break exits loop)
    assert!(
        interp.events.logs.len() >= 1,
        "expected at least one log event"
    );
    // If break didn't stop the loop we'd have many logs; ensure we have not 10 logs
    assert!(
        interp.events.logs.len() < 10,
        "break did not exit loop as expected"
    );

    Ok(())
}

#[test]
fn test_function_return_sets_return_variable() -> Result<()> {
    let mut interp = AudioInterpreter::new(44100);

    // Function body: return 42
    let ret_stmt = Statement::new(
        StatementKind::Return {
            value: Some(Box::new(Value::Number(42.0))),
        },
        Value::Null,
        0,
        2,
        1,
    );

    let func_stmt = Statement::new(
        StatementKind::Function {
            name: "myfunc".to_string(),
            parameters: vec![],
            body: vec![ret_stmt],
        },
        Value::Null,
        0,
        1,
        1,
    );

    // Call the function
    let call_stmt = Statement::new(
        StatementKind::Call {
            name: "myfunc".to_string(),
            args: vec![],
        },
        Value::Null,
        0,
        3,
        1,
    );

    interp.collect_events(&[func_stmt, call_stmt])?;

    // After execution, handler::handle_call stores returned value under "__return"
    if let Some(val) = interp.variables.get("__return") {
        match val {
            Value::Number(n) => assert_eq!(*n, 42.0),
            other => panic!("unexpected return type: {:?}", other),
        }
    } else {
        panic!("__return not set by function call");
    }

    Ok(())
}
