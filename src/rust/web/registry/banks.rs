//! Bank registry for WASM
//!
//! Manages registered audio banks and their triggers.
//! Banks are registered from JavaScript and made available to Devalang code.

use std::cell::RefCell;
use std::collections::HashMap;

/// Represents a registered audio bank
#[derive(Clone, Debug, serde::Serialize)]
pub struct RegisteredBank {
    /// Full name in format "publisher.name"
    pub full_name: String,
    /// Alias used in code (typically just "name")
    pub alias: String,
    /// Map of trigger names to relative paths
    /// Example: "kick" -> "audio/kick.wav"
    pub triggers: HashMap<String, String>,
}

thread_local! {
    /// Global registry of all registered banks
    pub static REGISTERED_BANKS: RefCell<Vec<RegisteredBank>> = RefCell::new(Vec::new());
}

/// Register a new bank
pub fn register_bank(full_name: String, alias: String, triggers: HashMap<String, String>) {
    REGISTERED_BANKS.with(|banks| {
        banks.borrow_mut().push(RegisteredBank {
            full_name,
            alias,
            triggers,
        });
    });
}

/// Clear all registered banks
pub fn clear_banks() {
    REGISTERED_BANKS.with(|banks| {
        banks.borrow_mut().clear();
    });
}

/// Get list of all registered banks
pub fn list_banks() -> Vec<RegisteredBank> {
    REGISTERED_BANKS.with(|banks| banks.borrow().clone())
}

/// Inject registered banks into interpreter
/// This makes banks available in Devalang code
pub fn inject_registered_banks(
    interpreter: &mut crate::engine::audio::interpreter::AudioInterpreter,
) {
    use crate::language::syntax::ast::Value;

    REGISTERED_BANKS.with(|banks| {
        for bank in banks.borrow().iter() {
            // Create a map for this bank with all triggers
            let mut trigger_map = HashMap::new();

            for (trigger_name, uri) in &bank.triggers {
                // The URI is already in the correct format: devalang://bank/{full_name}/{path}
                // Just use it directly
                trigger_map.insert(trigger_name.clone(), Value::String(uri.clone()));

                // Also store with _url suffix for backward compatibility
                trigger_map.insert(format!("{}_url", trigger_name), Value::String(uri.clone()));
            }

            // Ensure default properties exist on bank objects so dotted access like
            // `kit.kick` or `kit.volume` behave consistently at runtime.
            crate::utils::props::ensure_default_properties(&mut trigger_map, Some("trigger"));

            // Inject the bank map into interpreter variables
            // Example: kit = { kick: "devalang://bank/devaloop.808/kick", kick_url: "http://..." }
            interpreter
                .variables
                .insert(bank.alias.clone(), Value::Map(trigger_map));
        }
    });
}

/// Inject registered banks into a variable table (legacy)
/// This makes banks available as variables in Devalang code
pub fn inject_banks_into_globals(
    variables: &mut HashMap<String, crate::language::syntax::ast::Value>,
) {
    use crate::language::syntax::ast::Value;

    REGISTERED_BANKS.with(|banks| {
        for bank in banks.borrow().iter() {
            // Create a map for this bank with all triggers
            let mut trigger_map = HashMap::new();

            for (trigger_name, path) in &bank.triggers {
                // Store both the HTTP path and devalang:// URI
                trigger_map.insert(trigger_name.clone(), Value::String(path.clone()));

                // Also create devalang:// URI format
                let deva_uri = format!("devalang://bank/{}/{}", bank.full_name, path);
                trigger_map.insert(format!("{}_deva", trigger_name), Value::String(deva_uri));
            }

            // Ensure defaults for legacy-injected bank maps as well
            crate::utils::props::ensure_default_properties(&mut trigger_map, Some("trigger"));
            // Inject the bank map
            variables.insert(bank.alias.clone(), Value::Map(trigger_map));

            // Also inject individual triggers
            for (trigger_name, path) in &bank.triggers {
                let var_name = format!("{}.{}", bank.alias, trigger_name);
                variables.insert(var_name, Value::String(path.clone()));
            }
        }
    });
}
