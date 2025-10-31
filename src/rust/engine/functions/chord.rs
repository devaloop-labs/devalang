/// Chord function: plays multiple notes simultaneously
///
/// Usage: `synth -> chord([C4, E4, G4]) -> duration(800) -> velocity(100)`
/// Or: `synth -> chord(Cmaj7) -> duration(800)`
///
/// Arguments:
/// - notes: Array of note names or chord notation (e.g., "Cmaj7", "Dmin")
///
/// Chainable functions:
/// - duration(ms): Chord duration in milliseconds
/// - velocity(0-127): Chord velocity/volume
/// - pan(-1.0 to 1.0): Stereo panning (left to right)
/// - detune(-100 to 100): Pitch adjustment in cents
/// - spread(0.0-1.0): Stereo spread for chord notes
/// - gain(0.0-2.0): Volume multiplier
/// - attack(seconds): Attack time override
/// - release(seconds): Release time override
/// - strum(ms): Strum delay between notes
use super::{FunctionContext, FunctionExecutor};
use crate::language::syntax::ast::nodes::Value;
use anyhow::{Result, anyhow};

pub struct ChordFunction;

impl FunctionExecutor for ChordFunction {
    fn name(&self) -> &str {
        "chord"
    }

    fn execute(&self, context: &mut FunctionContext, args: &[Value]) -> Result<()> {
        if args.is_empty() {
            return Err(anyhow!(
                "chord() requires at least 1 argument (array of notes or chord name)"
            ));
        }

        // Parse notes - can be array or identifier (chord name like "Cmaj7")
        let notes = match &args[0] {
            Value::Array(arr) => arr
                .iter()
                .filter_map(|v| match v {
                    Value::String(s) | Value::Identifier(s) => Some(s.clone()),
                    _ => None,
                })
                .collect::<Vec<_>>(),
            Value::String(chord_name) | Value::Identifier(chord_name) => {
                // Try to parse as chord notation (e.g., "Cmaj7", "Dmin")
                parse_chord_notation(chord_name)?
            }
            _ => {
                return Err(anyhow!(
                    "chord() first argument must be an array of notes or chord name"
                ));
            }
        };

        if notes.is_empty() {
            return Err(anyhow!("chord() requires at least one note"));
        }

        // Store chord information - use "notes" key for clarity
        context.set(
            "notes",
            Value::Array(notes.iter().map(|n| Value::String(n.clone())).collect()),
        );

        Ok(())
    }
}

/// Helper to generate common chord types
#[allow(dead_code)]
pub fn generate_chord(root: &str, chord_type: &str) -> Result<Vec<String>> {
    use super::note::parse_note_to_midi;

    let root_midi = parse_note_to_midi(root)?;

    // Define intervals for different chord types
    let intervals = match chord_type.to_lowercase().as_str() {
        "major" | "maj" | "" => vec![0, 4, 7],  // Major triad
        "minor" | "min" | "m" => vec![0, 3, 7], // Minor triad
        "diminished" | "dim" => vec![0, 3, 6],  // Diminished
        "augmented" | "aug" => vec![0, 4, 8],   // Augmented
        "sus2" => vec![0, 2, 7],                // Suspended 2nd
        "sus4" => vec![0, 5, 7],                // Suspended 4th
        "7" | "dom7" => vec![0, 4, 7, 10],      // Dominant 7th
        "maj7" => vec![0, 4, 7, 11],            // Major 7th
        "min7" | "m7" => vec![0, 3, 7, 10],     // Minor 7th
        _ => return Err(anyhow!("Unknown chord type: {}", chord_type)),
    };

    // Generate notes
    let notes = intervals
        .iter()
        .map(|interval| midi_to_note(root_midi + interval))
        .collect::<Result<Vec<_>>>()?;

    Ok(notes)
}

#[allow(dead_code)]
fn midi_to_note(midi: u8) -> Result<String> {
    if midi > 127 {
        return Err(anyhow!("MIDI note out of range: {}", midi));
    }

    let octave = (midi / 12) as i32 - 1;
    let note_index = (midi % 12) as usize;

    let note_names = [
        "C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B",
    ];
    let note_name = note_names[note_index];

    Ok(format!("{}{}", note_name, octave))
}

/// Parse chord notation like "Cmaj7", "Dmin", "F#dim"
fn parse_chord_notation(chord_name: &str) -> Result<Vec<String>> {
    // Extract root note and chord type
    let chord_name = chord_name.trim();

    // Find where the note ends (after potential # or b)
    let mut root_end = 1;
    if chord_name.len() > 1 {
        let second_char = chord_name.chars().nth(1).unwrap();
        if second_char == '#' || second_char == 'b' {
            root_end = 2;
        }
    }

    // Extract chord type and octave
    // First check if rest looks like a chord type (to handle "G7" = dominant 7th, not G octave 7)
    let rest = &chord_name[root_end..];

    // Common chord type patterns: 7, maj7, min7, m7, dim, aug, sus2, sus4, etc.
    // If it starts with known chord type patterns, treat everything as chord type
    let is_chord_type = rest.starts_with("maj")
        || rest.starts_with("min")
        || rest.starts_with("dim")
        || rest.starts_with("aug")
        || rest.starts_with("sus")
        || rest == "7"
        || rest == "m"
        || rest.starts_with("m7")
        || rest == "+";

    let (root, chord_type) = if is_chord_type {
        // Everything after note is chord type, use default octave 4
        (format!("{}4", &chord_name[..root_end]), rest)
    } else {
        // Check for explicit octave
        if let Some(first_char) = rest.chars().next() {
            if first_char.is_ascii_digit() {
                // Has octave, extract it
                let octave_char = first_char;
                let type_rest = &rest[1..];
                (
                    format!("{}{}", &chord_name[..root_end], octave_char),
                    type_rest,
                )
            } else {
                // No octave, use default 4
                (format!("{}4", &chord_name[..root_end]), rest)
            }
        } else {
            // Empty rest, default octave and major
            (format!("{}4", &chord_name[..root_end]), "")
        }
    };

    // Normalize chord type
    let normalized_type = match chord_type.trim().to_lowercase().as_str() {
        "maj" | "major" | "" => "major",
        "min" | "minor" | "m" => "minor",
        "dim" | "diminished" => "diminished",
        "aug" | "augmented" | "+" => "augmented",
        "maj7" | "major7" => "maj7",
        "min7" | "minor7" | "m7" => "min7",
        "7" | "dom7" => "dom7",
        "sus2" => "sus2",
        "sus4" => "sus4",
        other => return Err(anyhow!("Unknown chord type: {}", other)),
    };

    generate_chord(&root, normalized_type)
}

#[cfg(test)]
#[path = "test_chord.rs"]
mod tests;
