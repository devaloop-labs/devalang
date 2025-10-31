/// Note function: plays a single note
///
/// Usage: `synth -> note(C4) -> velocity(100) -> duration(400)`
///
/// Arguments:
/// - note: String or Identifier (e.g., "C4", "D#5")
///
/// Chainable functions:
/// - duration(ms): Note duration in milliseconds
/// - velocity(0-127): Note velocity/volume
/// - pan(-1.0 to 1.0): Stereo panning (left to right)
/// - detune(-100 to 100): Pitch adjustment in cents
/// - gain(0.0-2.0): Volume multiplier
/// - attack(seconds): Attack time override
/// - release(seconds): Release time override
use super::{FunctionContext, FunctionExecutor};
use crate::language::syntax::ast::nodes::Value;
use anyhow::{Result, anyhow};

pub struct NoteFunction;

impl FunctionExecutor for NoteFunction {
    fn name(&self) -> &str {
        "note"
    }

    fn execute(&self, context: &mut FunctionContext, args: &[Value]) -> Result<()> {
        if args.is_empty() {
            return Err(anyhow!("note() requires at least 1 argument (note name)"));
        }

        // Parse note name
        let note_name = match &args[0] {
            Value::String(s) | Value::Identifier(s) => s.clone(),
            _ => return Err(anyhow!("note() first argument must be a note name")),
        };

        // Store note information in context
        context.set("note", Value::String(note_name.clone()));

        Ok(())
    }
}

/// Parse note name to MIDI number
/// C4 = 60, C#4 = 61, D4 = 62, etc.
pub fn parse_note_to_midi(note: &str) -> Result<u8> {
    let note = note.to_uppercase();

    // Extract base note (C, D, E, F, G, A, B)
    let base_char = note.chars().next().ok_or_else(|| anyhow!("Empty note"))?;
    let base_note = match base_char {
        'C' => 0,
        'D' => 2,
        'E' => 4,
        'F' => 5,
        'G' => 7,
        'A' => 9,
        'B' => 11,
        _ => return Err(anyhow!("Invalid note: {}", base_char)),
    };

    // Extract sharp/flat
    let mut offset = 0;
    let mut octave_start = 1;

    if note.len() > 1 {
        match note.chars().nth(1) {
            Some('#') => {
                offset = 1;
                octave_start = 2;
            }
            Some('b') | Some('B') => {
                offset = -1;
                octave_start = 2;
            }
            _ => {}
        }
    }

    // Extract octave
    let octave_str = &note[octave_start..];
    let octave: i32 = octave_str
        .parse()
        .map_err(|_| anyhow!("Invalid octave: {}", octave_str))?;

    // Calculate MIDI number: (octave + 1) * 12 + base_note + offset
    let midi = ((octave + 1) * 12) + base_note + offset;

    if midi < 0 || midi > 127 {
        return Err(anyhow!("MIDI note out of range: {}", midi));
    }

    Ok(midi as u8)
}

#[cfg(test)]
#[path = "test_note.rs"]
mod tests;
