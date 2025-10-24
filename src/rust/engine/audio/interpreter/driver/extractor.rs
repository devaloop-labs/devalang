use crate::language::syntax::ast::Value;
use anyhow::Result;
use crate::engine::audio::events::AudioEvent;

use super::AudioInterpreter;

/// Apply automation overrides to a parameter value
/// Returns the final value after applying global automation, then note-mode templates
/// For note-mode templates, calculates progress based on the note's start_time relative to the automation block
fn apply_automation_param(
    interpreter: &AudioInterpreter,
    target: &str,
    param_names: &[&str], // e.g., ["pan"] or ["pitch", "detune"]
    base_value: f32,
    note_start_time: f32, // Time when the note starts (used for note-mode progress calculation)
) -> f32 {
    let mut result = base_value;

    // 1. Try global automation first
    for name in param_names {
        if let Some(v) = interpreter
            .automation_registry
            .get_value(target, name, interpreter.cursor_time)
        {
            result = v;
            break; // Use first matching param name
        }
    }

    // 2. Apply note-mode templates
    if let Some(ctx) = interpreter.note_automation_templates.get(target) {
        // Calculate progress based on note's position in the automation block
        let note_progress = ctx.progress_at_time(note_start_time);
        
        for tpl in ctx.templates.iter() {
            for name in param_names {
                if tpl.param_name == *name {
                    // Evaluate template at the note's progress position
                    result = crate::engine::audio::automation::evaluate_template_at(tpl, note_progress);
                    return result; // Use first matching template
                }
            }
        }
    }

    result
}

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

        // Pan: preference order: explicit context > automation > default
        let base_pan = if let Some(Value::Number(p)) = context.get("pan") {
            *p
        } else {
            0.0
        };
        
        // Check if this note should use per-note automation
        let use_per_note_automation = interpreter.note_automation_templates.contains_key(target);
        
        // If using per-note automation, don't apply templates here - apply at render time
        let pan = if use_per_note_automation {
            base_pan
        } else {
            apply_automation_param(interpreter, target, &["pan"], base_pan, interpreter.cursor_time)
        };

        // Detune/Pitch: automation may come from either "pitch" or "detune" param name
        let base_detune = if let Some(Value::Number(d)) = context.get("detune") {
            *d
        } else {
            0.0
        };
        let detune = if use_per_note_automation {
            base_detune
        } else {
            apply_automation_param(interpreter, target, &["pitch", "detune"], base_detune, interpreter.cursor_time)
        };

        // Gain/Volume: automation may come from either "volume" or "gain" param name
        let base_gain = if let Some(Value::Number(g)) = context.get("gain") {
            *g
        } else {
            1.0
        };
        let gain = if use_per_note_automation {
            base_gain
        } else {
            apply_automation_param(interpreter, target, &["volume", "gain"], base_gain, interpreter.cursor_time)
        };

        // Use the provided target as synth id so the synth definition (including plugin info)
        // is correctly snapshotted at event creation time. Fall back to "default" if empty.
        let synth_id = if target.is_empty() { "default" } else { target };

        // Build a synth definition snapshot and apply automation overrides so that the
        // snapshot used for this specific event reflects current automations.
        let mut synth_def = interpreter
            .events
            .get_synth(synth_id)
            .cloned()
            .unwrap_or_default();

        // Apply automation overrides to synth options (cutoff, resonance, etc.)
        // Note: cutoff and resonance are in filters, not options
        for filter in &mut synth_def.filters {
            // Apply cutoff automation
            let current_cutoff = filter.cutoff;
            let automated_cutoff = apply_automation_param(interpreter, synth_id, &["cutoff"], current_cutoff, interpreter.cursor_time);
            if (automated_cutoff - current_cutoff).abs() > 0.0001 {
                filter.cutoff = automated_cutoff;
            }
            
            // Apply resonance automation
            let current_resonance = filter.resonance;
            let automated_resonance = apply_automation_param(interpreter, synth_id, &["resonance"], current_resonance, interpreter.cursor_time);
            if (automated_resonance - current_resonance).abs() > 0.0001 {
                filter.resonance = automated_resonance;
            }
        }

        // Apply automation overrides to synth type options (drive, tone, decay, etc.)
        let synth_params = ["filter_type", "drive", "tone", "decay"];
        for param in &synth_params {
            let current_val = synth_def.options.get(*param).copied().unwrap_or(0.0);
            let automated_val = apply_automation_param(interpreter, synth_id, &[param], current_val, interpreter.cursor_time);
            if (automated_val - current_val).abs() > 0.0001 {
                synth_def.options.insert(param.to_string(), automated_val);
            }
        }

        // Log resolved values for diagnostics
        #[cfg(feature = "cli")]
        {
            crate::tools::logger::Logger::new().info(format!(
                "Scheduling Note: synth='{}' time={} dur={} pan={} detune={} gain={}",
                synth_id,
                interpreter.cursor_time,
                duration,
                pan,
                detune,
                gain
            ));
        }

        // Create the AudioEvent::Note directly so we can control the synth snapshot used
        use crate::engine::audio::events::AudioEvent;
        interpreter.events.events.push(AudioEvent::Note {
            midi,
            start_time: interpreter.cursor_time,
            duration,
            velocity,
            synth_id: synth_id.to_string(),
            synth_def,
            pan,
            detune,
            gain,
            attack: None,
            release: None,
            delay_time: None,
            delay_feedback: None,
            delay_mix: None,
            reverb_amount: None,
            drive_amount: None,
            drive_color: None,
            use_per_note_automation,
        });
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
            
            // Check if this chord should use per-note automation
            let use_per_note_automation = interpreter.note_automation_templates.contains_key(target);
            
            let pan = if use_per_note_automation {
                pan
            } else {
                apply_automation_param(interpreter, target, &["pan"], pan, interpreter.cursor_time)
            };

            let detune = if let Some(Value::Number(d)) = context.get("detune") {
                *d
            } else {
                0.0
            };
            let detune = if use_per_note_automation {
                detune
            } else {
                apply_automation_param(interpreter, target, &["pitch", "detune"], detune, interpreter.cursor_time)
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
            let gain = if use_per_note_automation {
                gain
            } else {
                apply_automation_param(interpreter, target, &["volume", "gain"], gain, interpreter.cursor_time)
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

            // Create chord event directly with per-note automation flag
            let synth_def = interpreter.events.get_synth(synth_id).cloned().unwrap_or_default();
            interpreter.events.events.push(AudioEvent::Chord {
                midis,
                start_time: interpreter.cursor_time,
                duration,
                velocity,
                synth_id: synth_id.to_string(),
                synth_def,
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
                use_per_note_automation,
            });
            // Apply note-mode/global automation to synth-specific options (cutoff, resonance, etc.)
            // Collect automated values first to avoid borrowing conflicts
            let synth_params = ["cutoff", "resonance", "filter_type", "drive", "tone", "decay"];
            let mut param_updates = Vec::new();
            
            // Snapshot current values and compute automations
            if let Some(synth_def) = interpreter.events.synths.get(synth_id) {
                for param in &synth_params {
                    let current_val = synth_def.options.get(*param).copied().unwrap_or(0.0);
                    let automated_val = apply_automation_param(interpreter, synth_id, &[param], current_val, interpreter.cursor_time);
                    if (automated_val - current_val).abs() > 0.0001 {
                        param_updates.push((param.to_string(), automated_val));
                    }
                }
            }
            
            // Now apply the updates
            if let Some(synth_def) = interpreter.events.synths.get_mut(synth_id) {
                for (param, val) in param_updates {
                    synth_def.options.insert(param, val);
                }
            }
        }
    }
    Ok(())
}
