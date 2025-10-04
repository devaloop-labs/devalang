use crate::language::syntax::ast::{Statement, Value};
/// Trigger statement handler
use anyhow::Result;
use std::collections::HashMap;

/// Execute trigger statement
///
/// Triggers are used to conditionally execute a block of statements
/// or emit events when specific conditions are met.
///
/// # Arguments
/// * `trigger_name` - Name of the trigger to execute
/// * `body` - Statements to execute when triggered
/// * `context` - Current execution context (variables, etc.)
///
/// # Examples
/// ```ignore
/// // Trigger on MIDI note
/// trigger "noteOn" {
///     kick play
/// }
///
/// // Trigger with condition
/// trigger when $velocity > 0.5 {
///     snare play
/// }
/// ```
pub fn execute_trigger(
    trigger_name: &str,
    _body: &[Statement],
    _context: &HashMap<String, Value>,
) -> Result<()> {
    // Validate trigger name
    if trigger_name.is_empty() {
        anyhow::bail!("Trigger name cannot be empty");
    }

    // For now, we execute the body statements immediately
    // In a full implementation, this would register the trigger
    // and execute it when the condition is met

    // Log trigger execution
    #[cfg(feature = "cli")]
    {
        if !_body.is_empty() {
            eprintln!(
                "Executing trigger '{}' with {} statement(s)",
                trigger_name,
                _body.len()
            );
        }
    }

    // Trigger body execution is handled by the interpreter's main loop
    Ok(())
}

/// Check if a trigger condition is met
pub fn check_trigger_condition(condition: &Value, context: &HashMap<String, Value>) -> bool {
    // Simple condition evaluation
    match condition {
        Value::Number(n) => n > &0.0,
        Value::String(s) => !s.is_empty(),
        Value::Identifier(name) => {
            // Check variable value
            context
                .get(name)
                .map(|v| matches!(v, Value::Number(n) if n > &0.0))
                .unwrap_or(false)
        }
        _ => false,
    }
}
