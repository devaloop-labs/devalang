use anyhow::Result;

use crate::engine::audio::interpreter::driver::AudioInterpreter;
use crate::language::syntax::ast::{DurationValue, Statement, StatementKind, Value};
use std::collections::HashMap;

#[test]
fn test_break_in_loop_exits_early() -> Result<()> {
    let mut interp = AudioInterpreter::new(44100);

    // Ensure a minimal bank variable exists so `.bank.kick` resolves to a sample URI
    let mut bank_map = std::collections::HashMap::new();
    bank_map.insert(
        "kick".to_string(),
        Value::String("file://kick.wav".to_string()),
    );
    interp
        .variables
        .insert("bank".to_string(), Value::Map(bank_map));

    // Create a loop of 10 iterations but body contains a trigger then a break
    let trigger = Statement::trigger(".bank.kick", DurationValue::Milliseconds(100.0), None, 1, 1);
    let break_stmt = Statement::new(StatementKind::Break, Value::Null, 0, 2, 1);

    let loop_stmt = Statement::new(
        StatementKind::Loop {
            count: Value::Number(10.0),
            body: vec![trigger, break_stmt],
        },
        Value::Null,
        0,
        1,
        1,
    );

    // Collect events
    interp.collect_events(&[loop_stmt])?;

    // Only the first trigger should have been scheduled (break exits loop)
    assert!(
        interp.events.events.len() >= 1,
        "expected at least one event"
    );
    // If break didn't stop the loop we'd have many events; ensure we have not 10 events
    assert!(
        interp.events.events.len() < 10,
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

#[test]
fn test_nested_break_inner_only() -> Result<()> {
    let mut interp = AudioInterpreter::new(44100);

    // Inner loop will do a print then break; outer loop runs 3 times and should see 3 prints
    let inner_print = Statement::print("inner", 1, 1);
    let inner_break = Statement::new(StatementKind::Break, Value::Null, 0, 2, 1);
    let inner_loop = Statement::new(
        StatementKind::Loop {
            count: Value::Number(10.0),
            body: vec![inner_print, inner_break],
        },
        Value::Null,
        0,
        1,
        1,
    );

    let outer_loop = Statement::new(
        StatementKind::Loop {
            count: Value::Number(3.0),
            body: vec![inner_loop],
        },
        Value::Null,
        0,
        1,
        1,
    );

    interp.collect_events(&[outer_loop])?;

    // Expect exactly 3 inner prints (one per outer iteration)
    assert_eq!(
        interp.events.logs.len(),
        3,
        "expected 3 inner prints, got {}",
        interp.events.logs.len()
    );

    Ok(())
}

#[test]
fn test_pass_ms_loop_exits_on_break() -> Result<()> {
    let mut interp = AudioInterpreter::new(44100);

    // Loop pass(1) with a print + break in body; should execute once and exit
    let p = Statement::print("tick", 1, 1);
    let b = Statement::new(StatementKind::Break, Value::Null, 0, 2, 1);
    let pass_call = Value::Call {
        name: "pass".to_string(),
        args: vec![Value::Number(1.0)],
    };

    let loop_stmt = Statement::new(
        StatementKind::Loop {
            count: pass_call,
            body: vec![p, b],
        },
        Value::Null,
        0,
        1,
        1,
    );

    interp.collect_events(&[loop_stmt])?;
    assert!(interp.events.logs.len() >= 1, "expected at least one log");
    assert!(
        interp.events.logs.len() < 10,
        "pass(ms) loop did not exit on break"
    );

    Ok(())
}

#[test]
fn test_return_nested_function_propagation() -> Result<()> {
    let mut interp = AudioInterpreter::new(44100);

    // inner returns 7
    let inner_ret = Statement::new(
        StatementKind::Return {
            value: Some(Box::new(Value::Number(7.0))),
        },
        Value::Null,
        0,
        2,
        1,
    );
    let inner_func = Statement::new(
        StatementKind::Function {
            name: "inner".to_string(),
            parameters: vec![],
            body: vec![inner_ret],
        },
        Value::Null,
        0,
        1,
        1,
    );

    // outer calls inner, then returns __return (value set by inner)
    let call_inner = Statement::new(
        StatementKind::Call {
            name: "inner".to_string(),
            args: vec![],
        },
        Value::Null,
        0,
        3,
        1,
    );
    let outer_ret = Statement::new(
        StatementKind::Return {
            value: Some(Box::new(Value::Identifier("__return".to_string()))),
        },
        Value::Null,
        0,
        4,
        1,
    );
    let outer_func = Statement::new(
        StatementKind::Function {
            name: "outer".to_string(),
            parameters: vec![],
            body: vec![call_inner, outer_ret],
        },
        Value::Null,
        0,
        5,
        1,
    );

    let call_outer = Statement::new(
        StatementKind::Call {
            name: "outer".to_string(),
            args: vec![],
        },
        Value::Null,
        0,
        6,
        1,
    );

    interp.collect_events(&[inner_func, outer_func, call_outer])?;

    if let Some(v) = interp.variables.get("__return") {
        if let Value::Number(n) = v {
            assert_eq!(*n, 7.0);
        } else {
            panic!("unexpected __return type");
        }
    } else {
        panic!("__return missing after nested function call");
    }

    Ok(())
}

#[test]
fn test_post_increment_indexing() -> Result<()> {
    let mut interp = AudioInterpreter::new(44100);

    // Set i = 0, arr = [10,20]
    interp.variables.insert("i".to_string(), Value::Number(0.0));
    interp.variables.insert(
        "arr".to_string(),
        Value::Array(vec![Value::Number(10.0), Value::Number(20.0)]),
    );

    // Print arr[i++]
    let print_idx = Statement::new(
        StatementKind::Print,
        Value::Identifier("arr[i++]".to_string()),
        0,
        1,
        1,
    );
    interp.collect_events(&[print_idx])?;

    // Expect one log with "10" and i incremented to 1
    assert_eq!(interp.events.logs.len(), 1);
    let msg = &interp.events.logs[0].1;
    assert!(
        msg.contains("10"),
        "expected printed value to contain 10, got {}",
        msg
    );
    if let Some(Value::Number(n)) = interp.variables.get("i") {
        assert_eq!(*n as i32, 1);
    } else {
        panic!("i missing or wrong type");
    }

    Ok(())
}

#[test]
fn test_arithmetic_indexing() -> Result<()> {
    let mut interp = AudioInterpreter::new(44100);

    // arr = [10,20,30], i = 0
    interp.variables.insert("i".to_string(), Value::Number(0.0));
    interp.variables.insert(
        "arr".to_string(),
        Value::Array(vec![
            Value::Number(10.0),
            Value::Number(20.0),
            Value::Number(30.0),
        ]),
    );

    // print arr[i+1] and arr[i*2]
    let p1 = Statement::new(
        StatementKind::Print,
        Value::Identifier("arr[i+1]".to_string()),
        0,
        1,
        1,
    );
    let p2 = Statement::new(
        StatementKind::Print,
        Value::Identifier("arr[i*2]".to_string()),
        0,
        1,
        2,
    );
    interp.collect_events(&[p1, p2])?;

    // logs should contain 20 and 10 or 30 depending on evaluation
    assert!(interp.events.logs.len() >= 2);
    let m1 = &interp.events.logs[0].1;
    let m2 = &interp.events.logs[1].1;
    assert!(m1.contains("20") || m1.contains("20.0"));
    assert!(m2.contains("10") || m2.contains("10.0") || m2.contains("30") || m2.contains("30.0"));

    Ok(())
}

#[test]
fn test_effect_chaining_on_trigger() -> Result<()> {
    let mut interp = AudioInterpreter::new(44100);

    // Register a sample variable
    interp.variables.insert(
        "kick".to_string(),
        Value::String("file://kick.wav".to_string()),
    );

    // Build effects chain: [{ "type": "reverse", "enabled": true }, { "type": "speed", "factor": 1.5 }]
    let mut e1 = HashMap::new();
    e1.insert("type".to_string(), Value::String("reverse".to_string()));
    e1.insert("enabled".to_string(), Value::Boolean(true));

    let mut e2 = HashMap::new();
    e2.insert("type".to_string(), Value::String("speed".to_string()));
    e2.insert("factor".to_string(), Value::Number(1.5));

    let effects = Value::Array(vec![Value::Map(e1.clone()), Value::Map(e2.clone())]);

    // Trigger with effects
    let trigger = Statement::trigger(
        "kick",
        DurationValue::Milliseconds(100.0),
        Some(effects.clone()),
        1,
        1,
    );
    interp.collect_events(&[trigger])?;

    // Find a Sample event and check its effects field
    let sample_events: Vec<_> = interp
        .events
        .events
        .iter()
        .filter_map(|e| match e {
            crate::engine::audio::events::AudioEvent::Sample {
                uri: _,
                start_time: _,
                velocity: _,
                effects,
            } => Some(effects.clone()),
            _ => None,
        })
        .collect();

    assert!(
        !sample_events.is_empty(),
        "expected at least one sample event"
    );
    let first_effects = &sample_events[0];
    match first_effects {
        Some(Value::Array(arr)) => {
            assert_eq!(arr.len(), 2);
        }
        other => panic!("unexpected effects payload: {:?}", other),
    }

    Ok(())
}
