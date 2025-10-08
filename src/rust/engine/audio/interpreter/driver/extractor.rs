use anyhow::Result;
use crate::language::syntax::ast::Value;

use super::AudioInterpreter;

pub fn extract_audio_event(interpreter: &mut AudioInterpreter, target: &str, context: &crate::engine::functions::FunctionContext) -> Result<()> {
    // Implementation simplified: reuse existing driver logic
    if let Some(Value::String(note_name)) = context.get("note") {
        let midi = crate::engine::functions::note::parse_note_to_midi(note_name)?;
        let duration = if let Some(Value::Number(d)) = context.get("duration") { d / 1000.0 } else { 0.5 };
        let velocity = if let Some(Value::Number(v)) = context.get("velocity") { v / 100.0 } else { 0.8 };
        let pan = if let Some(Value::Number(p)) = context.get("pan") { *p } else { 0.0 };
        let detune = if let Some(Value::Number(d)) = context.get("detune") { *d } else { 0.0 };
        let gain = if let Some(Value::Number(g)) = context.get("gain") { *g } else { 1.0 };

        // Use the provided target as synth id so the synth definition (including plugin info)
        // is correctly snapshotted at event creation time. Fall back to "default" if empty.
        let synth_id = if target.is_empty() { "default" } else { target };
        interpreter.events.add_note_event(synth_id, midi, interpreter.cursor_time, duration, velocity, pan, detune, gain, None, None, None, None, None, None, None, None);
    }
    Ok(())
}
