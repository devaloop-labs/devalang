use crate::engine::audio::interpreter::driver::AudioInterpreter;
use crate::language::syntax::ast::{Statement, StatementKind, Value};

#[test]
fn test_loop_empty_body_stops() {
    let mut it = AudioInterpreter::new(44100);
    // body with only a print -> should not produce audio and should stop quickly
    let body = vec![Statement::print("hello", 1, 1)];
    let res = it.execute_loop(&Value::Null, &body);
    assert!(res.is_ok());
    assert!(
        it.events.events.is_empty(),
        "No events should be produced for empty body"
    );
}

#[test]
fn test_loop_produces_audio_then_drains() {
    let mut it = AudioInterpreter::new(44100);
    // Set up a sample variable so `spawn s` resolves to a sample and produces audio
    it.variables
        .insert("s".to_string(), Value::String("sample://foo".to_string()));

    // Build statements: trigger 's' once using a guard variable, then clear the guard so
    // subsequent iterations produce no audio and the loop can drain.
    it.variables
        .insert("do_once".to_string(), Value::Boolean(true));

    let trigger = Statement::trigger(
        "s",
        crate::language::syntax::ast::DurationValue::Auto,
        None,
        1,
        1,
    );

    let if_stmt = Statement::new(
        StatementKind::If {
            condition: Value::Identifier("do_once".to_string()),
            body: vec![trigger.clone()],
            else_body: None,
        },
        Value::Null,
        0,
        2,
        1,
    );

    let let_stmt = Statement::new(
        StatementKind::Let {
            name: "do_once".to_string(),
            value: Some(Value::Boolean(false)),
        },
        Value::Null,
        0,
        3,
        1,
    );

    let body = vec![if_stmt, let_stmt];

    let res = it.execute_loop(&Value::Null, &body);
    assert!(res.is_ok());
    // After execution, there should be at least one event produced
    assert!(it.events.events.len() > 0, "Expected events to be produced");
    // And total duration should be >= cursor_time (events extend beyond cursor at least for first spawn)
    assert!(it.events.total_duration() >= it.cursor_time);
}
