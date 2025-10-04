use once_cell::sync::Lazy;
use std::collections::HashSet;

static BOOLEAN_LITERALS: Lazy<HashSet<&'static str>> =
    Lazy::new(|| ["true", "false"].into_iter().collect());

pub fn is_boolean_literal(value: &str) -> bool {
    BOOLEAN_LITERALS.contains(&value.to_ascii_lowercase().as_str())
}

pub fn is_duration_literal(value: &str) -> bool {
    let value = value.trim().to_ascii_lowercase();
    value.ends_with("ms") || value.ends_with('b') || value == "auto"
}
