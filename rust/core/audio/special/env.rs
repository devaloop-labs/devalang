use devalang_types::Value;

use crate::core::store::variable::VariableTable;
use std::sync::OnceLock;
use std::time::{SystemTime, UNIX_EPOCH};

static SESSION_SEED: OnceLock<f32> = OnceLock::new();

pub fn get_session_seed() -> f32 {
    *SESSION_SEED.get_or_init(|| {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default();
        // Build a stable 0..1 seed from nanos
        let nanos = now.subsec_nanos();
        ((nanos as f32) / 1_000_000_000.0).clamp(0.0, 1.0)
    })
}

// Resolve special environment variables like $env.bpm, $env.beat, $env.position
// For now, $env.position is treated as an alias of beat.
pub fn resolve_env_atom(atom: &str, bpm: f32, beat: f32) -> Option<f32> {
    match atom {
        "$env.bpm" => Some(bpm),
        "$env.beat" => Some(beat),
        "$env.position" => Some(beat),
        // Optional seed for deterministic randomness
        "$env.seed" => Some(get_session_seed()),
        _ => None,
    }
}

// Utility: resolve an identifier or numeric literal to f32 using the variable table
pub fn resolve_atom_or_var(atom: &str, vars: &VariableTable, bpm: f32, beat: f32) -> Option<f32> {
    if let Some(v) = resolve_env_atom(atom, bpm, beat) {
        return Some(v);
    }
    if let Ok(n) = atom.parse::<f32>() {
        return Some(n);
    }
    if let Some(Value::Number(n)) = vars.get(atom) {
        return Some(*n);
    }
    None
}
