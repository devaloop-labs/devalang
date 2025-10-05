/// Event system for Devalang - "on" and "emit" statements
/// Allows reactive programming and event-driven audio generation
use crate::language::syntax::ast::{Statement, Value};
use std::collections::HashMap;

/// Event handler - function called when event is emitted
#[derive(Debug, Clone)]
pub struct EventHandler {
    pub event_name: String,
    pub body: Vec<Statement>,
    pub once: bool, // If true, handler runs only once
}

/// Event payload - data associated with an emitted event
#[derive(Debug, Clone)]
pub struct EventPayload {
    pub event_name: String,
    pub data: HashMap<String, Value>,
    pub timestamp: f32, // When the event was emitted
}

/// Event registry - manages event handlers and emitted events
#[derive(Debug, Clone, Default)]
pub struct EventRegistry {
    handlers: HashMap<String, Vec<EventHandler>>,
    emitted_events: Vec<EventPayload>,
    executed_once: HashMap<String, usize>, // Track which handlers have executed (for "once")
}

impl EventRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    /// Register an event handler
    pub fn register_handler(&mut self, handler: EventHandler) {
        let event_name = handler.event_name.clone();
        self.handlers
            .entry(event_name)
            .or_insert_with(Vec::new)
            .push(handler);
    }

    /// Emit an event with optional data
    pub fn emit(&mut self, event_name: String, data: HashMap<String, Value>, timestamp: f32) {
        let payload = EventPayload {
            event_name,
            data,
            timestamp,
        };
        self.emitted_events.push(payload);
    }

    /// Get handlers for a specific event
    pub fn get_handlers(&self, event_name: &str) -> Vec<EventHandler> {
        self.handlers.get(event_name).cloned().unwrap_or_default()
    }

    /// Get all handlers matching a pattern (supports wildcards)
    pub fn get_handlers_matching(&self, pattern: &str) -> Vec<EventHandler> {
        let mut matching = Vec::new();

        for (event_name, handlers) in &self.handlers {
            if pattern_matches(pattern, event_name) {
                matching.extend(handlers.clone());
            }
        }

        matching
    }

    /// Check if a "once" handler should be executed
    pub fn should_execute_once(&mut self, event_name: &str, handler_index: usize) -> bool {
        let key = format!("{}:{}", event_name, handler_index);

        if self.executed_once.contains_key(&key) {
            return false;
        }

        self.executed_once.insert(key, 1);
        true
    }

    /// Get all emitted events
    pub fn get_emitted_events(&self) -> &[EventPayload] {
        &self.emitted_events
    }

    /// Get emitted events for a specific event name
    pub fn get_events_by_name(&self, event_name: &str) -> Vec<EventPayload> {
        self.emitted_events
            .iter()
            .filter(|e| e.event_name == event_name)
            .cloned()
            .collect()
    }

    /// Clear all emitted events (useful for new playback runs)
    pub fn clear_events(&mut self) {
        self.emitted_events.clear();
        self.executed_once.clear();
    }

    /// Get number of handlers for an event
    pub fn handler_count(&self, event_name: &str) -> usize {
        self.handlers.get(event_name).map(|h| h.len()).unwrap_or(0)
    }

    /// Remove all handlers for an event
    pub fn remove_handlers(&mut self, event_name: &str) {
        self.handlers.remove(event_name);
    }
}

/// Check if a pattern matches an event name
/// Supports wildcards: * (any characters), ? (single character)
fn pattern_matches(pattern: &str, event_name: &str) -> bool {
    if pattern == "*" {
        return true;
    }

    if !pattern.contains('*') && !pattern.contains('?') {
        return pattern == event_name;
    }

    // Convert pattern to regex-like matching
    let mut pattern_chars = pattern.chars().peekable();
    let mut name_chars = event_name.chars().peekable();

    while pattern_chars.peek().is_some() || name_chars.peek().is_some() {
        match pattern_chars.peek() {
            Some('*') => {
                pattern_chars.next();

                // If * is at the end, match everything remaining
                if pattern_chars.peek().is_none() {
                    return true;
                }

                // Try to match remaining pattern with remaining name
                let remaining_pattern: String = pattern_chars.clone().collect();
                while name_chars.peek().is_some() {
                    let remaining_name: String = name_chars.clone().collect();
                    if pattern_matches(&remaining_pattern, &remaining_name) {
                        return true;
                    }
                    name_chars.next();
                }
                return false;
            }
            Some('?') => {
                pattern_chars.next();
                if name_chars.next().is_none() {
                    return false;
                }
            }
            Some(p) => {
                let p = *p;
                pattern_chars.next();
                match name_chars.next() {
                    Some(n) if n == p => continue,
                    _ => return false,
                }
            }
            None => {
                return name_chars.peek().is_none();
            }
        }
    }

    true
}

/// Built-in event types
pub mod builtin_events {
    pub const BEAT: &str = "beat";
    pub const BAR: &str = "bar";
    pub const START: &str = "start";
    pub const END: &str = "end";
    pub const TEMPO_CHANGE: &str = "tempo.change";
    pub const NOTE_ON: &str = "note.on";
    pub const NOTE_OFF: &str = "note.off";
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_register_and_get_handler() {
        let mut registry = EventRegistry::new();

        let handler = EventHandler {
            event_name: "test".to_string(),
            body: vec![],
            once: false,
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
        });

        registry.register_handler(EventHandler {
            event_name: "note.off".to_string(),
            body: vec![],
            once: false,
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
}
