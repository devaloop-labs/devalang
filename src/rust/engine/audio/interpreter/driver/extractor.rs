use crate::language::syntax::ast::Value;
use anyhow::Result;

use super::AudioInterpreter;

pub fn extract_audio_event(
    interpreter: &mut AudioInterpreter,
    target: &str,
    context: &crate::engine::functions::FunctionContext,
) -> Result<()> {
    // Implementation simplified: reuse existing driver logic
    // Try single note first
    if let Some(Value::String(note_name)) = context.get("note") {
        let midi = crate::engine::functions::note::parse_note_to_midi(note_name)?;

        // Duration: prefer context.duration if set (execute_arrow_call may set it), otherwise read "duration" value (ms)
        let duration = if context.duration > 0.0 {
            context.duration
        } else if let Some(Value::Number(d)) = context.get("duration") {
            d / 1000.0
        } else {
            0.5
        };

        // Velocity: accept either 0.0-1.0 or 0-127 (if > 2.0 treat as MIDI scale)
        let velocity = if let Some(Value::Number(v)) = context.get("velocity") {
            if *v > 2.0 {
                (*v) / 127.0
            } else if *v > 1.0 {
                (*v) / 100.0
            } else {
                *v
            }
        } else {
            0.8
        };

        let pan = if let Some(Value::Number(p)) = context.get("pan") {
            *p
        } else {
            0.0
        };
        let detune = if let Some(Value::Number(d)) = context.get("detune") {
            *d
        } else {
            0.0
        };
        let gain = if let Some(Value::Number(g)) = context.get("gain") {
            *g
        } else {
            1.0
        };

        // Use the provided target as synth id so the synth definition (including plugin info)
        // is correctly snapshotted at event creation time. Fall back to "default" if empty.
        let synth_id = if target.is_empty() { "default" } else { target };
        interpreter.events.add_note_event(
            synth_id,
            midi,
            interpreter.cursor_time,
            duration,
            velocity,
            pan,
            detune,
            gain,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        );
        return Ok(());
    }

    // Handle chords (notes array)
    if let Some(Value::Array(notes_arr)) = context.get("notes") {
        let mut midis: Vec<u8> = Vec::new();
        for n in notes_arr {
            if let Value::String(s) = n {
                if let Ok(m) = crate::engine::functions::note::parse_note_to_midi(s) {
                    midis.push(m);
                }
            }
        }

        if !midis.is_empty() {
            let duration = if context.duration > 0.0 {
                context.duration
            } else if let Some(Value::Number(d)) = context.get("duration") {
                d / 1000.0
            } else {
                0.5
            };

            let velocity = if let Some(Value::Number(v)) = context.get("velocity") {
                if *v > 2.0 {
                    (*v) / 127.0
                } else if *v > 1.0 {
                    (*v) / 100.0
                } else {
                    *v
                }
            } else {
                0.8
            };

            let pan = if let Some(Value::Number(p)) = context.get("pan") {
                *p
            } else {
                0.0
            };
            let detune = if let Some(Value::Number(d)) = context.get("detune") {
                *d
            } else {
                0.0
            };
            let spread = if let Some(Value::Number(s)) = context.get("spread") {
                *s
            } else {
                0.0
            };
            let gain = if let Some(Value::Number(g)) = context.get("gain") {
                *g
            } else {
                1.0
            };

            // optional envelope overrides
            let attack = context.get("attack").and_then(|v| {
                if let Value::Number(n) = v {
                    Some(*n)
                } else {
                    None
                }
            });
            let release = context.get("release").and_then(|v| {
                if let Value::Number(n) = v {
                    Some(*n)
                } else {
                    None
                }
            });

            // Effects
            let delay_time = context.get("delay_time").and_then(|v| {
                if let Value::Number(n) = v {
                    Some(*n)
                } else {
                    None
                }
            });
            let delay_feedback = context.get("delay_feedback").and_then(|v| {
                if let Value::Number(n) = v {
                    Some(*n)
                } else {
                    None
                }
            });
            let delay_mix = context.get("delay_mix").and_then(|v| {
                if let Value::Number(n) = v {
                    Some(*n)
                } else {
                    None
                }
            });
            let reverb_amount = context.get("reverb_amount").and_then(|v| {
                if let Value::Number(n) = v {
                    Some(*n)
                } else {
                    None
                }
            });
            let drive_amount = context.get("drive_amount").and_then(|v| {
                if let Value::Number(n) = v {
                    Some(*n)
                } else {
                    None
                }
            });
            let drive_color = context.get("drive_color").and_then(|v| {
                if let Value::Number(n) = v {
                    Some(*n)
                } else {
                    None
                }
            });

            let synth_id = if target.is_empty() { "default" } else { target };

            interpreter.events.add_chord_event(
                synth_id,
                midis,
                interpreter.cursor_time,
                duration,
                velocity,
                pan,
                detune,
                spread,
                gain,
                attack,
                release,
                delay_time,
                delay_feedback,
                delay_mix,
                reverb_amount,
                drive_amount,
                drive_color,
            );
        }
    }
    Ok(())
}
