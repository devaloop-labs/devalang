use super::*;

#[test]
fn test_register_and_get_handler() {
    let mut registry = EventRegistry::new();

    let handler = EventHandler {
        event_name: "test".to_string(),
        body: vec![],
        once: false,
        args: None,
    };

    registry.register_handler(handler);

    let handlers = registry.get_handlers("test");
    assert_eq!(handlers.len(), 1);
    assert_eq!(handlers[0].event_name, "test");
}

#[test]
fn test_emit_and_get_events() {
    let mut registry = EventRegistry::new();

    let mut data = HashMap::new();
    data.insert("value".to_string(), Value::Number(42.0));

    registry.emit("test".to_string(), data, 0.0);

    let events = registry.get_events_by_name("test");
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].event_name, "test");
}

#[test]
fn test_pattern_matches() {
    assert!(pattern_matches("*", "anything"));
    assert!(pattern_matches("test", "test"));
    assert!(!pattern_matches("test", "other"));
    assert!(pattern_matches("test*", "test123"));
    assert!(pattern_matches("*test", "mytest"));
    assert!(pattern_matches("te?t", "test"));
    assert!(!pattern_matches("te?t", "teast"));
}

#[test]
fn test_once_handler() {
    let mut registry = EventRegistry::new();

    // First execution should return true
    assert!(registry.should_execute_once("test", 0));

    // Second execution should return false
    assert!(!registry.should_execute_once("test", 0));
}

#[test]
fn test_wildcard_handlers() {
    let mut registry = EventRegistry::new();

    registry.register_handler(EventHandler {
        event_name: "note.on".to_string(),
        body: vec![],
        once: false,
        args: None,
    });

    registry.register_handler(EventHandler {
        event_name: "note.off".to_string(),
        body: vec![],
        once: false,
        args: None,
    });

    let handlers = registry.get_handlers_matching("note.*");
    assert_eq!(handlers.len(), 2);
}

#[test]
fn test_clear_events() {
    let mut registry = EventRegistry::new();

    registry.emit("test".to_string(), HashMap::new(), 0.0);
    assert_eq!(registry.get_emitted_events().len(), 1);

    registry.clear_events();
    assert_eq!(registry.get_emitted_events().len(), 0);
}

#[test]
fn test_on_beat_interval_executes_every_n() {
    use crate::engine::audio::interpreter::driver::AudioInterpreter;
    use crate::language::syntax::ast::{Statement, StatementKind, Value};

    let mut interp = AudioInterpreter::new(44100);

    // Handler body emits 'fired'
    let emit_stmt = Statement {
        kind: StatementKind::Emit {
            event: "fired".to_string(),
            payload: None,
        },
        value: Value::Null,
        indent: 0,
        line: 0,
        column: 0,
    };

    let handler = EventHandler {
        event_name: "beat".to_string(),
        body: vec![emit_stmt],
        once: false,
        args: Some(vec![Value::Number(2.0)]),
    };

    interp.event_registry.register_handler(handler);

    // At beat 0 -> should execute
    interp.special_vars.current_beat = 0.0;
    interp.event_registry.clear_events();
    interp.suppress_beat_emit = true;
    let _ = crate::engine::audio::interpreter::driver::handler::execute_event_handlers(
        &mut interp,
        "beat",
    );
    assert_eq!(interp.event_registry.get_emitted_events().len(), 1);

    // At beat 1 -> should not execute
    interp.event_registry.clear_events();
    interp.special_vars.current_beat = 1.0;
    interp.suppress_beat_emit = true;
    let _ = crate::engine::audio::interpreter::driver::handler::execute_event_handlers(
        &mut interp,
        "beat",
    );
    assert_eq!(interp.event_registry.get_emitted_events().len(), 0);
}

#[test]
fn test_on_bar_interval_executes_every_n() {
    use crate::engine::audio::interpreter::driver::AudioInterpreter;
    use crate::language::syntax::ast::{Statement, StatementKind, Value};

    let mut interp = AudioInterpreter::new(44100);

    let emit_stmt = Statement {
        kind: StatementKind::Emit {
            event: "barred".to_string(),
            payload: None,
        },
        value: Value::Null,
        indent: 0,
        line: 0,
        column: 0,
    };

    let handler = EventHandler {
        event_name: "bar".to_string(),
        body: vec![emit_stmt],
        once: false,
        args: Some(vec![Value::Number(2.0)]),
    };

    interp.event_registry.register_handler(handler);

    // bar 0 -> executes
    interp.special_vars.current_bar = 0.0;
    interp.event_registry.clear_events();
    interp.suppress_beat_emit = true;
    let _ = crate::engine::audio::interpreter::driver::handler::execute_event_handlers(
        &mut interp,
        "bar",
    );
    assert_eq!(interp.event_registry.get_emitted_events().len(), 1);

    // bar 1 -> does not execute
    interp.event_registry.clear_events();
    interp.special_vars.current_bar = 1.0;
    interp.suppress_beat_emit = true;
    let _ = crate::engine::audio::interpreter::driver::handler::execute_event_handlers(
        &mut interp,
        "bar",
    );
    assert_eq!(interp.event_registry.get_emitted_events().len(), 0);
}
