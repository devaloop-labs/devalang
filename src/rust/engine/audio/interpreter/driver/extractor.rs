use crate::engine::audio::events::AudioEvent;
use crate::language::syntax::ast::Value;
use anyhow::Result;

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
        if let Some(v) =
            interpreter
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
                    result =
                        crate::engine::audio::automation::evaluate_template_at(tpl, note_progress);
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
        // Prepare placeholder for merged effects (will be computed after collecting synth/note effects)
        let mut event_effects: Option<crate::language::syntax::ast::Value> = None;
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
            apply_automation_param(
                interpreter,
                target,
                &["pan"],
                base_pan,
                interpreter.cursor_time,
            )
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
            apply_automation_param(
                interpreter,
                target,
                &["pitch", "detune"],
                base_detune,
                interpreter.cursor_time,
            )
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
            apply_automation_param(
                interpreter,
                target,
                &["volume", "gain"],
                base_gain,
                interpreter.cursor_time,
            )
        };

        // Use the provided target as synth id so the synth definition (including plugin info)
        // is correctly snapshotted at event creation time. Fall back to "default" if empty.
        let synth_id = if target.is_empty() { "default" } else { target };

        // Build a synth definition snapshot and apply automation overrides so that the

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
            let automated_cutoff = apply_automation_param(
                interpreter,
                synth_id,
                &["cutoff"],
                current_cutoff,
                interpreter.cursor_time,
            );
            if (automated_cutoff - current_cutoff).abs() > 0.0001 {
                filter.cutoff = automated_cutoff;
            }

            // Apply resonance automation
            let current_resonance = filter.resonance;
            let automated_resonance = apply_automation_param(
                interpreter,
                synth_id,
                &["resonance"],
                current_resonance,
                interpreter.cursor_time,
            );
            if (automated_resonance - current_resonance).abs() > 0.0001 {
                filter.resonance = automated_resonance;
            }
        }

        // Apply automation overrides to synth type options (drive, tone, decay, etc.)
        let synth_params = ["filter_type", "drive", "tone", "decay"];
        for param in &synth_params {
            let current_val = synth_def.options.get(*param).copied().unwrap_or(0.0);
            let automated_val = apply_automation_param(
                interpreter,
                synth_id,
                &[param],
                current_val,
                interpreter.cursor_time,
            );
            if (automated_val - current_val).abs() > 0.0001 {
                synth_def.options.insert(param.to_string(), automated_val);
            }
        }

        let mut synth_effects_vec: Vec<crate::language::syntax::ast::Value> = Vec::new();
        if let Some(var_val) = interpreter.variables.get(synth_id) {
            match var_val {
                crate::language::syntax::ast::Value::Map(m) => {
                    if let Some(chain_v) = m.get("chain") {
                        if let crate::language::syntax::ast::Value::Array(arr) = chain_v {
                            synth_effects_vec.extend(arr.clone());
                        }
                    } else if let Some(effs) = m.get("effects") {
                        // Deprecated map-style: normalize into individual effect maps
                        eprintln!(
                            "DEPRECATION: synth-level effect param map support is deprecated — use chained params instead."
                        );
                        let normalized =
                            crate::engine::audio::effects::normalize_effects(&Some(effs.clone()));
                        for (k, v) in normalized.into_iter() {
                            let mut map = std::collections::HashMap::new();
                            map.insert(
                                "type".to_string(),
                                crate::language::syntax::ast::Value::String(k),
                            );
                            for (pk, pv) in v.into_iter() {
                                map.insert(pk, pv);
                            }
                            synth_effects_vec.push(crate::language::syntax::ast::Value::Map(map));
                        }
                    }
                }
                crate::language::syntax::ast::Value::Statement(stmt_box) => {
                    if let crate::language::syntax::ast::StatementKind::ArrowCall { .. } =
                        &stmt_box.kind
                    {
                        if let crate::language::syntax::ast::Value::Map(m) = &stmt_box.value {
                            if let Some(crate::language::syntax::ast::Value::Array(arr)) =
                                m.get("chain")
                            {
                                synth_effects_vec.extend(arr.clone());
                            }
                        }
                    }
                }
                _ => {}
            }
        }

        // Collect note-level effects from FunctionContext when invoking note/chord/sample
        let mut note_effects_vec: Vec<crate::language::syntax::ast::Value> = Vec::new();
        if let Some(crate::language::syntax::ast::Value::String(method_name)) =
            context.get("method")
        {
            let m = method_name.as_str();
            if m == "note" || m == "chord" || m == "sample" {
                if let Some(eff_val) = context.get("effects") {
                    match eff_val {
                        crate::language::syntax::ast::Value::Array(arr) => {
                            note_effects_vec.extend(arr.clone());
                        }
                        crate::language::syntax::ast::Value::Map(map_v) => {
                            // normalize map into per-effect entries
                            let normalized = crate::engine::audio::effects::normalize_effects(
                                &Some(crate::language::syntax::ast::Value::Map(map_v.clone())),
                            );
                            for (k, v) in normalized.into_iter() {
                                let mut map = std::collections::HashMap::new();
                                map.insert(
                                    "type".to_string(),
                                    crate::language::syntax::ast::Value::String(k),
                                );
                                for (pk, pv) in v.into_iter() {
                                    map.insert(pk, pv);
                                }
                                note_effects_vec
                                    .push(crate::language::syntax::ast::Value::Map(map));
                            }
                        }
                        _ => {}
                    }
                }
            }
        }

        // As a last resort, if no synth variable chain and context.effects exists (non-note invocation), treat context.effects as synth-level
        if synth_effects_vec.is_empty() {
            if let Some(eff_val) = context.get("effects") {
                if let crate::language::syntax::ast::Value::Array(arr) = eff_val {
                    synth_effects_vec.extend(arr.clone());
                } else if let crate::language::syntax::ast::Value::Map(map_v) = eff_val {
                    let normalized = crate::engine::audio::effects::normalize_effects(&Some(
                        crate::language::syntax::ast::Value::Map(map_v.clone()),
                    ));
                    for (k, v) in normalized.into_iter() {
                        let mut map = std::collections::HashMap::new();
                        map.insert(
                            "type".to_string(),
                            crate::language::syntax::ast::Value::String(k),
                        );
                        for (pk, pv) in v.into_iter() {
                            map.insert(pk, pv);
                        }
                        synth_effects_vec.push(crate::language::syntax::ast::Value::Map(map));
                    }
                }
            }
        }

        // Merge synth-level then note-level
        if !synth_effects_vec.is_empty() || !note_effects_vec.is_empty() {
            let mut merged: Vec<crate::language::syntax::ast::Value> = Vec::new();
            merged.extend(synth_effects_vec);
            merged.extend(note_effects_vec);
            event_effects = Some(crate::language::syntax::ast::Value::Array(merged));
        }

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
            effects: event_effects,
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
            let use_per_note_automation =
                interpreter.note_automation_templates.contains_key(target);

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
                apply_automation_param(
                    interpreter,
                    target,
                    &["pitch", "detune"],
                    detune,
                    interpreter.cursor_time,
                )
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
                apply_automation_param(
                    interpreter,
                    target,
                    &["volume", "gain"],
                    gain,
                    interpreter.cursor_time,
                )
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
            let synth_def = interpreter
                .events
                .get_synth(synth_id)
                .cloned()
                .unwrap_or_default();
            // Determine effects for this chord event by merging synth-level and any chord-level effects
            let mut event_effects: Option<crate::language::syntax::ast::Value> = None;

            // Collect synth-level effects if present in variable
            let mut synth_effects_vec: Vec<crate::language::syntax::ast::Value> = Vec::new();
            if let Some(var_val) = interpreter.variables.get(synth_id) {
                match var_val {
                    crate::language::syntax::ast::Value::Map(m) => {
                        if let Some(chain_v) = m.get("chain") {
                            if let crate::language::syntax::ast::Value::Array(arr) = chain_v {
                                synth_effects_vec.extend(arr.clone());
                            }
                        } else if let Some(effs) = m.get("effects") {
                            eprintln!(
                                "DEPRECATION: synth-level effect param map support is deprecated — use chained params instead."
                            );
                            let normalized = crate::engine::audio::effects::normalize_effects(
                                &Some(effs.clone()),
                            );
                            for (k, v) in normalized.into_iter() {
                                let mut map = std::collections::HashMap::new();
                                map.insert(
                                    "type".to_string(),
                                    crate::language::syntax::ast::Value::String(k),
                                );
                                for (pk, pv) in v.into_iter() {
                                    map.insert(pk, pv);
                                }
                                synth_effects_vec
                                    .push(crate::language::syntax::ast::Value::Map(map));
                            }
                        }
                    }
                    crate::language::syntax::ast::Value::Statement(stmt_box) => {
                        if let crate::language::syntax::ast::StatementKind::ArrowCall { .. } =
                            &stmt_box.kind
                        {
                            if let crate::language::syntax::ast::Value::Map(m) = &stmt_box.value {
                                if let Some(crate::language::syntax::ast::Value::Array(arr)) =
                                    m.get("chain")
                                {
                                    synth_effects_vec.extend(arr.clone());
                                }
                            }
                        }
                    }
                    _ => {}
                }
            }

            // Collect chord-level (note-level) effects from FunctionContext when invoking chord
            let mut chord_effects_vec: Vec<crate::language::syntax::ast::Value> = Vec::new();
            if let Some(crate::language::syntax::ast::Value::String(method_name)) =
                context.get("method")
            {
                let m = method_name.as_str();
                if m == "chord" {
                    if let Some(eff_val) = context.get("effects") {
                        match eff_val {
                            crate::language::syntax::ast::Value::Array(arr) => {
                                chord_effects_vec.extend(arr.clone());
                            }
                            crate::language::syntax::ast::Value::Map(map_v) => {
                                let normalized = crate::engine::audio::effects::normalize_effects(
                                    &Some(crate::language::syntax::ast::Value::Map(map_v.clone())),
                                );
                                for (k, v) in normalized.into_iter() {
                                    let mut map = std::collections::HashMap::new();
                                    map.insert(
                                        "type".to_string(),
                                        crate::language::syntax::ast::Value::String(k),
                                    );
                                    for (pk, pv) in v.into_iter() {
                                        map.insert(pk, pv);
                                    }
                                    chord_effects_vec
                                        .push(crate::language::syntax::ast::Value::Map(map));
                                }
                            }
                            _ => {}
                        }
                    }
                }
            }

            if !synth_effects_vec.is_empty() || !chord_effects_vec.is_empty() {
                let mut merged: Vec<crate::language::syntax::ast::Value> = Vec::new();
                merged.extend(synth_effects_vec);
                merged.extend(chord_effects_vec);
                event_effects = Some(crate::language::syntax::ast::Value::Array(merged));
            }

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
                effects: event_effects,
                use_per_note_automation: false,
            });
            // Apply note-mode/global automation to synth-specific options (cutoff, resonance, etc.)
            // Collect automated values first to avoid borrowing conflicts
            let synth_params = [
                "cutoff",
                "resonance",
                "filter_type",
                "drive",
                "tone",
                "decay",
            ];
            let mut param_updates = Vec::new();

            // Snapshot current values and compute automations
            if let Some(synth_def) = interpreter.events.synths.get(synth_id) {
                for param in &synth_params {
                    let current_val = synth_def.options.get(*param).copied().unwrap_or(0.0);
                    let automated_val = apply_automation_param(
                        interpreter,
                        synth_id,
                        &[param],
                        current_val,
                        interpreter.cursor_time,
                    );
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
