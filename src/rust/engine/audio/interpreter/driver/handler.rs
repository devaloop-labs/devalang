// Full handler implementation copied from top-level module
use anyhow::Result;
use std::collections::HashMap;

use crate::language::syntax::ast::{Value, Statement, StatementKind};

use super::AudioInterpreter;

pub fn handle_let(interpreter: &mut AudioInterpreter, name: &str, value: &Value) -> Result<()> {
    // Check if this is a synth definition (has waveform parameter OR _plugin_ref)
    if let Value::Map(map) = value {
        if map.contains_key("waveform") || map.contains_key("_plugin_ref") {
            // plugin handling simplified: reuse existing logic
            let mut is_plugin = false;
            let mut plugin_author: Option<String> = None;
            let mut plugin_name: Option<String> = None;
            let mut plugin_export: Option<String> = None;

            if let Some(Value::String(plugin_ref)) = map.get("_plugin_ref") {
                let parts: Vec<&str> = plugin_ref.split('.').collect();
                if parts.len() == 2 {
                    let (var_name, prop_name) = (parts[0], parts[1]);
                    if let Some(var_value) = interpreter.variables.get(var_name) {
                        if let Value::Map(var_map) = var_value {
                            if let Some(Value::String(resolved_plugin)) = var_map.get(prop_name) {
                                if resolved_plugin.starts_with("plugin:") {
                                    let ref_parts: Vec<&str> = resolved_plugin["plugin:".len()..].split(':').collect();
                                    if ref_parts.len() == 2 {
                                        let full_plugin_name = ref_parts[0];
                                        let export_name = ref_parts[1];
                                        let plugin_parts: Vec<&str> = full_plugin_name.split('.').collect();
                                        if plugin_parts.len() == 2 {
                                            plugin_author = Some(plugin_parts[0].to_string());
                                            plugin_name = Some(plugin_parts[1].to_string());
                                            plugin_export = Some(export_name.to_string());
                                            is_plugin = true;
                                        }
                                    }
                                }
                            } else if let Some(Value::Map(export_map)) = var_map.get(prop_name) {
                                if let (Some(Value::String(author)), Some(Value::String(name)), Some(Value::String(export))) = (
                                    export_map.get("_plugin_author"),
                                    export_map.get("_plugin_name"),
                                    export_map.get("_export_name")
                                ) {
                                    plugin_author = Some(author.clone());
                                    plugin_name = Some(name.clone());
                                    plugin_export = Some(export.clone());
                                    is_plugin = true;
                                }
                            }
                        }
                    }
                }
            }

            let waveform = crate::engine::audio::events::extract_string(map, "waveform", "sine");
            let attack = crate::engine::audio::events::extract_number(map, "attack", 0.01);
            let decay = crate::engine::audio::events::extract_number(map, "decay", 0.1);
            let sustain = crate::engine::audio::events::extract_number(map, "sustain", 0.7);
            let release = crate::engine::audio::events::extract_number(map, "release", 0.2);

            let synth_type = if let Some(Value::String(t)) = map.get("type") {
                let clean = t.trim_matches('"').trim_matches('\'');
                if clean.is_empty() || clean == "synth" { None } else { Some(clean.to_string()) }
            } else { None };

            let filters = if let Some(Value::Array(filters_arr)) = map.get("filters") {
                crate::engine::audio::events::extract_filters(filters_arr)
            } else { Vec::new() };

            let mut options = std::collections::HashMap::new();
            let reserved_keys = if is_plugin {
                vec!["attack", "decay", "sustain", "release", "type", "filters", "_plugin_ref"]
            } else {
                vec!["waveform", "attack", "decay", "sustain", "release", "type", "filters", "_plugin_ref"]
            };

            for (key, val) in map.iter() {
                if !reserved_keys.contains(&key.as_str()) {
                    match val {
                        Value::Number(n) => { options.insert(key.clone(), *n); }
                        Value::String(s) => {
                            if is_plugin && key == "waveform" {
                                let waveform_id = match s.trim_matches('"').trim_matches('\'').to_lowercase().as_str() {
                                    "sine" => 0.0,
                                    "saw" => 1.0,
                                    "square" => 2.0,
                                    "triangle" => 3.0,
                                    _ => 1.0,
                                };
                                options.insert(key.clone(), waveform_id);
                            }
                        }
                        _ => {}
                    }
                }
            }

            if is_plugin && map.contains_key("decay") {
                options.insert("decay".to_string(), decay);
            }

            let final_waveform = if is_plugin { "plugin".to_string() } else { waveform };

            let synth_def = crate::engine::audio::events::SynthDefinition {
                waveform: final_waveform,
                attack,
                decay,
                sustain,
                release,
                synth_type,
                filters,
                options,
                plugin_author,
                plugin_name,
                plugin_export,
            };

            // Log plugin info for diagnostics: why plugin path may not be used later
            println!("🎹 Synth registered: {} -> plugin_author={:?}, plugin_name={:?}, plugin_export={:?}", name, synth_def.plugin_author, synth_def.plugin_name, synth_def.plugin_export);
            interpreter.events.add_synth(name.to_string(), synth_def);
        }
    }

    interpreter.variables.insert(name.to_string(), value.clone());
    Ok(())
}

pub fn handle_call(interpreter: &mut AudioInterpreter, name: &str) -> Result<()> {
    // Check inline pattern or pattern variable or group call
    // Clone the variable value first to avoid holding an immutable borrow across a mutable call
    if let Some(pattern_value) = interpreter.variables.get(name).cloned() {
        if let Value::Statement(stmt_box) = pattern_value {
            if let StatementKind::Pattern { target, .. } = &stmt_box.kind {
                if let Some(tgt) = target.as_ref() {
                        let (pattern_str, options) = interpreter.extract_pattern_data(&stmt_box.value);
                        if let Some(pat) = pattern_str {
                            println!("🎵 Call pattern: {} with {}", name, tgt);
                            interpreter.execute_pattern(tgt.as_str(), &pat, options)?;
                            return Ok(());
                        }
                    }
            }
        }
    }

    if let Some(body) = interpreter.groups.get(name).cloned() {
        println!("📞 Call group: {}", name);
        super::collector::collect_events(interpreter, &body)?;
    } else {
        println!("⚠️  Warning: Group or pattern '{}' not found", name);
    }

    Ok(())
}

pub fn execute_print(interpreter: &AudioInterpreter, value: &Value) -> Result<()> {
    let message = match value {
        Value::String(s) => {
            if s.contains('{') && s.contains('}') { interpreter.interpolate_string(s) } else { s.clone() }
        }
        Value::Identifier(id) => {
            // Resolve variable from interpreter.variables
            if let Some(v) = interpreter.variables.get(id) {
                match v {
                    Value::String(s) => s.clone(),
                    Value::Number(n) => n.to_string(),
                    Value::Boolean(b) => b.to_string(),
                    Value::Array(arr) => format!("{:?}", arr),
                    Value::Map(map) => format!("{:?}", map),
                    _ => format!("{:?}", v),
                }
            } else {
                format!("Identifier(\"{}\")", id)
            }
        }
        Value::Number(n) => n.to_string(),
        Value::Boolean(b) => b.to_string(),
        Value::Array(arr) => format!("{:?}", arr),
        Value::Map(map) => format!("{:?}", map),
        _ => format!("{:?}", value),
    };

    println!("💬 {}", message);
    Ok(())
}

pub fn execute_if(
    interpreter: &mut AudioInterpreter,
    condition: &Value,
    body: &[Statement],
    else_body: &Option<Vec<Statement>>,
) -> Result<()> {
    let condition_result = interpreter.evaluate_condition(condition)?;

    if condition_result {
        super::collector::collect_events(interpreter, body)?;
    } else if let Some(else_stmts) = else_body {
        super::collector::collect_events(interpreter, else_stmts)?;
    }

    Ok(())
}

pub fn execute_event_handlers(interpreter: &mut AudioInterpreter, event_name: &str) -> Result<()> {
    let handlers = interpreter.event_registry.get_handlers_matching(event_name);

    for (index, handler) in handlers.iter().enumerate() {
        if handler.once && !interpreter.event_registry.should_execute_once(event_name, index) {
            continue;
        }

        let body_clone = handler.body.clone();
        super::collector::collect_events(interpreter, &body_clone)?;
    }

    Ok(())
}

pub fn handle_assign(interpreter: &mut AudioInterpreter, target: &str, property: &str, value: &Value) -> Result<()> {
    if let Some(var) = interpreter.variables.get_mut(target) {
        if let Value::Map(map) = var {
            map.insert(property.to_string(), value.clone());

            if interpreter.events.synths.contains_key(target) {
                let map_clone = map.clone();
                let updated_def = interpreter.extract_synth_def_from_map(&map_clone)?;
                interpreter.events.synths.insert(target.to_string(), updated_def);
                println!("🔧 Updated {}.{} = {:?}", target, property, value);
            }
        } else {
            return Err(anyhow::anyhow!("Cannot assign property '{}' to non-map variable '{}'", property, target));
        }
    } else {
        return Err(anyhow::anyhow!("Variable '{}' not found", target));
    }

    Ok(())
}

pub fn extract_synth_def_from_map(interpreter: &AudioInterpreter, map: &HashMap<String, Value>) -> Result<crate::engine::audio::events::SynthDefinition> {
    use crate::engine::audio::events::extract_filters;

    let waveform = crate::engine::audio::events::extract_string(map, "waveform", "sine");
    let attack = crate::engine::audio::events::extract_number(map, "attack", 0.01);
    let decay = crate::engine::audio::events::extract_number(map, "decay", 0.1);
    let sustain = crate::engine::audio::events::extract_number(map, "sustain", 0.7);
    let release = crate::engine::audio::events::extract_number(map, "release", 0.2);

    let synth_type = if let Some(Value::String(t)) = map.get("type") {
        let clean = t.trim_matches('"').trim_matches('\'');
        if clean.is_empty() || clean == "synth" { None } else { Some(clean.to_string()) }
    } else { None };

    let filters = if let Some(Value::Array(filters_arr)) = map.get("filters") { extract_filters(filters_arr) } else { Vec::new() };

    let plugin_author = if let Some(Value::String(s)) = map.get("plugin_author") { Some(s.clone()) } else { None };
    let plugin_name = if let Some(Value::String(s)) = map.get("plugin_name") { Some(s.clone()) } else { None };
    let plugin_export = if let Some(Value::String(s)) = map.get("plugin_export") { Some(s.clone()) } else { None };

    let mut options = HashMap::new();
    for (key, val) in map.iter() {
        if ![
            "waveform", "attack", "decay", "sustain", "release", "type", "filters",
            "plugin_author", "plugin_name", "plugin_export",
        ].contains(&key.as_str())
        {
            if let Value::Number(n) = val {
                options.insert(key.clone(), *n);
            } else if let Value::String(s) = val {
                if key == "waveform" || key.starts_with("_") { continue; }
                if let Ok(n) = s.parse::<f32>() { options.insert(key.clone(), n); }
            }
        }
    }

    Ok(crate::engine::audio::events::SynthDefinition {
        waveform,
        attack,
        decay,
        sustain,
        release,
        synth_type,
        filters,
        options,
        plugin_author,
        plugin_name,
        plugin_export,
    })
}

pub fn handle_load(interpreter: &mut AudioInterpreter, source: &str, alias: &str) -> Result<()> {
    use std::path::Path;

    let path = Path::new(source);
    // Determine extension
    if let Some(ext) = path.extension().and_then(|s| s.to_str()).map(|s| s.to_lowercase()) {
        match ext.as_str() {
            "mid" | "midi" => {
                use crate::engine::audio::midi::load_midi_file;
                let midi_data = load_midi_file(path)?;
                interpreter.variables.insert(alias.to_string(), midi_data);
                println!("🎵 Loaded MIDI file: {} as {}", source, alias);
                Ok(())
            }
            "wav" | "flac" | "mp3" | "ogg" => {
                // For now support WAV via existing parser; other formats may be supported later.
                use crate::engine::audio::samples;
                let registered = samples::register_sample_from_path(path)?;
                // Record the sample URI under the alias variable as a string (consistent with triggers)
                interpreter.variables.insert(alias.to_string(), Value::String(registered.clone()));
                println!("🎵 Loaded sample file: {} as {} (uri={})", source, alias, registered);
                Ok(())
            }
            _ => Err(anyhow::anyhow!("Unsupported file type for @load: {}", ext)),
        }
    } else {
        Err(anyhow::anyhow!("Cannot determine file extension for {}", source))
    }
}

pub fn handle_bind(interpreter: &mut AudioInterpreter, source: &str, target: &str, options: &Value) -> Result<()> {
    let midi_data = interpreter.variables.get(source).ok_or_else(|| anyhow::anyhow!("MIDI source '{}' not found", source))?.clone();

    if let Value::Map(midi_map) = &midi_data {
        let notes = midi_map.get("notes").ok_or_else(|| anyhow::anyhow!("MIDI data has no notes"))?;

        if let Value::Array(notes_array) = notes {
            let _synth_def = interpreter.events.synths.get(target).ok_or_else(|| anyhow::anyhow!("Synth '{}' not found", target))?.clone();

            let default_velocity = 100;
            let mut velocity = default_velocity;

            if let Value::Map(opts) = options {
                if let Some(Value::Number(v)) = opts.get("velocity") { velocity = *v as u8; }
            }

            // Determine MIDI file BPM (if present) so we can rescale times to interpreter BPM
            // Default to interpreter.bpm when the MIDI file has no BPM metadata
            let midi_bpm = crate::engine::audio::events::extract_number(midi_map, "bpm", interpreter.bpm);

            for note_val in notes_array {
                if let Value::Map(note_map) = note_val {
                    let time = crate::engine::audio::events::extract_number(note_map, "time", 0.0);
                    let note = crate::engine::audio::events::extract_number(note_map, "note", 60.0) as u8;
                    let note_velocity = crate::engine::audio::events::extract_number(note_map, "velocity", velocity as f32) as u8;
                    // Duration may be present (ms) from MIDI loader; fallback to 500 ms
                    let duration_ms = crate::engine::audio::events::extract_number(note_map, "duration", 500.0);

                    use crate::engine::audio::events::AudioEvent;
                    let synth_def = interpreter.events.get_synth(target).cloned().unwrap_or_default();
                    // Rescale times according to interpreter BPM vs MIDI file BPM.
                    // If midi_bpm == interpreter.bpm this is a no-op. We compute factor = midi_bpm / interpreter.bpm
                    let interp_bpm = interpreter.bpm;
                    let factor = if interp_bpm > 0.0 { midi_bpm / interp_bpm } else { 1.0 };

                    let start_time_s = (time / 1000.0) * factor;
                    let duration_s = (duration_ms / 1000.0) * factor;

                    let event = AudioEvent::Note {
                        midi: note,
                        start_time: start_time_s,
                        duration: duration_s,
                        velocity: note_velocity as f32,
                        synth_id: target.to_string(),
                        synth_def,
                        pan: 0.0,
                        detune: 0.0,
                        gain: 1.0,
                        attack: None,
                        release: None,
                        delay_time: None,
                        delay_feedback: None,
                        delay_mix: None,
                        reverb_amount: None,
                        drive_amount: None,
                        drive_color: None,
                    };

                    // Diagnostic: log each scheduled note from bind (midi, time ms, start_time sec)
                    println!("🔔 bind_note -> midi={} time_ms={:.0} start_time_s={:.3} synth={}", note, time, time / 1000.0, target);
                    interpreter.events.events.push(event);
                }
            }

            println!("🔗 Bound {} notes from {} to {}", notes_array.len(), source, target);
        }
    }

    Ok(())
}

pub fn handle_use_plugin(interpreter: &mut AudioInterpreter, author: &str, name: &str, alias: &str) -> Result<()> {
    use crate::engine::plugin::loader::load_plugin;

    match load_plugin(author, name) {
        Ok((info, _wasm_bytes)) => {
            let mut plugin_map = HashMap::new();
            plugin_map.insert("_type".to_string(), Value::String("plugin".to_string()));
            plugin_map.insert("_author".to_string(), Value::String(info.author.clone()));
            plugin_map.insert("_name".to_string(), Value::String(info.name.clone()));

            if let Some(version) = &info.version { plugin_map.insert("_version".to_string(), Value::String(version.clone())); }

            for export in &info.exports {
                let mut export_map = HashMap::new();
                export_map.insert("_plugin_author".to_string(), Value::String(info.author.clone()));
                export_map.insert("_plugin_name".to_string(), Value::String(info.name.clone()));
                export_map.insert("_export_name".to_string(), Value::String(export.name.clone()));
                export_map.insert("_export_kind".to_string(), Value::String(export.kind.clone()));

                plugin_map.insert(export.name.clone(), Value::Map(export_map));
            }

            interpreter.variables.insert(alias.to_string(), Value::Map(plugin_map));
            println!("🔌 Plugin loaded: {}.{} as {}", author, name, alias);
        }
        Err(e) => {
            eprintln!("❌ Failed to load plugin {}.{}: {}", author, name, e);
            return Err(anyhow::anyhow!("Failed to load plugin: {}", e));
        }
    }

    Ok(())
}

pub fn handle_bank(interpreter: &mut AudioInterpreter, name: &str, alias: &Option<String>) -> Result<()> {
    let target_alias = alias.clone().unwrap_or_else(|| name.split('.').last().unwrap_or(name).to_string());

    if let Some(existing_value) = interpreter.variables.get(name) {
        interpreter.variables.insert(target_alias.clone(), existing_value.clone());
        #[cfg(not(feature = "wasm"))]
        println!("🏦 Bank alias created: {} -> {}", name, target_alias);
    } else {
        #[cfg(feature = "wasm")]
        {
            use crate::web::registry::banks::REGISTERED_BANKS;
            REGISTERED_BANKS.with(|banks| {
                for bank in banks.borrow().iter() {
                            if bank.full_name == *name {
                        if let Some(Value::Map(bank_map)) = interpreter.variables.get(&bank.alias) {
                            interpreter.variables.insert(target_alias.clone(), Value::Map(bank_map.clone()));
                        }
                    }
                }
            });
        }

        #[cfg(not(feature = "wasm"))]
        {
            if let Ok(current_dir) = std::env::current_dir() {
                match interpreter.banks.register_bank(target_alias.clone(), &name, &current_dir, &current_dir) {
                    Ok(_) => {
                        let mut bank_map = HashMap::new();
                        bank_map.insert("_name".to_string(), Value::String(name.to_string()));
                        bank_map.insert("_alias".to_string(), Value::String(target_alias.clone()));
                        interpreter.variables.insert(target_alias.clone(), Value::Map(bank_map));
                        println!("🏦 Bank registered: {} as {}", name, target_alias);
                    }
                    Err(e) => {
                        eprintln!("⚠️ Failed to register bank '{}': {}", name, e);
                        let mut bank_map = HashMap::new();
                        bank_map.insert("_name".to_string(), Value::String(name.to_string()));
                        bank_map.insert("_alias".to_string(), Value::String(target_alias.clone()));
                        interpreter.variables.insert(target_alias.clone(), Value::Map(bank_map));
                    }
                }
            } else {
                let mut bank_map = HashMap::new();
                bank_map.insert("_name".to_string(), Value::String(name.to_string()));
                bank_map.insert("_alias".to_string(), Value::String(target_alias.clone()));
                interpreter.variables.insert(target_alias.clone(), Value::Map(bank_map));
                eprintln!("⚠️ Could not determine cwd to register bank '{}', registered minimal alias.", name);
            }
        }
    }

    #[cfg(not(feature = "wasm"))]
    println!("🏦 Bank handling completed for {} (alias {}).", name, target_alias);

    Ok(())
}

pub fn handle_trigger(interpreter: &mut AudioInterpreter, entity: &str) -> Result<()> {
    let resolved_entity = if entity.starts_with('.') { &entity[1..] } else { entity };

    if resolved_entity.contains('.') {
        let parts: Vec<&str> = resolved_entity.split('.').collect();
        if parts.len() == 2 {
            let (var_name, property) = (parts[0], parts[1]);

            if let Some(Value::Map(map)) = interpreter.variables.get(var_name) {
                if let Some(Value::String(sample_uri)) = map.get(property) {
                    let uri = sample_uri.trim_matches('"').trim_matches('\'');
                    println!("🎵 Trigger: {}.{} -> {} at {:.3}s", var_name, property, uri, interpreter.cursor_time);
                    interpreter.events.add_sample_event(uri, interpreter.cursor_time, 1.0);
                    let beat_duration = interpreter.beat_duration();
                    interpreter.cursor_time += beat_duration;
                        } else {
                    #[cfg(not(feature = "wasm"))]
                    {
                        println!("🔍 Tentative de résolution du déclencheur: {}", resolved_entity);
                        // First try to produce an internal devalang://bank URI (preferred, supports lazy loading)
                        let resolved_uri = interpreter.resolve_sample_uri(resolved_entity);
                        if resolved_uri != resolved_entity {
                            println!("🎵 Résolution via variable: {} -> {}", resolved_entity, resolved_uri);
                            interpreter.events.add_sample_event(&resolved_uri, interpreter.cursor_time, 1.0);
                            let beat_duration = interpreter.beat_duration();
                            interpreter.cursor_time += beat_duration;
                        } else if let Some(pathbuf) = interpreter.banks.resolve_trigger(var_name, property) {
                            if let Some(path_str) = pathbuf.to_str() {
                                println!("🎵 Résolution réussie (fichier): {}.{} -> {}", var_name, property, path_str);
                                interpreter.events.add_sample_event(path_str, interpreter.cursor_time, 1.0);
                                let beat_duration = interpreter.beat_duration();
                                interpreter.cursor_time += beat_duration;
                            } else {
                                println!("⚠️ Résolution échouée pour {}.{} (path invalide)", var_name, property);
                            }
                        } else {
                            println!("⚠️ Aucun chemin trouvé pour {} via BankRegistry", resolved_entity);
                        }
                    }
                }
            }
        }
    } else {
        if let Some(Value::String(sample_uri)) = interpreter.variables.get(resolved_entity) {
            let uri = sample_uri.trim_matches('"').trim_matches('\'');
            println!("🎵 Trigger: {} -> {} at {:.3}s", resolved_entity, uri, interpreter.cursor_time);
            interpreter.events.add_sample_event(uri, interpreter.cursor_time, 1.0);
            let beat_duration = interpreter.beat_duration();
            interpreter.cursor_time += beat_duration;
        }
    }

    println!("🔄 Déclencheur interprété: {}", entity);
    #[cfg(not(feature = "wasm"))]
    {
        println!("🔍 Déclencheurs disponibles dans BankRegistry:");
        for (bank_name, bank) in interpreter.banks.list_banks() {
            println!("   Banque: {}", bank_name);
            for trigger in bank.list_triggers() {
                println!("      Déclencheur: {}", trigger);
            }
        }
    }

    // Note: do not call interpreter.render_audio() here - rendering is handled by the build pipeline.
    println!("🎵 Trigger queued for rendering: {} (events collected)", entity);

    Ok(())
}

pub fn extract_pattern_data(_interpreter: &AudioInterpreter, value: &Value) -> (Option<String>, Option<HashMap<String, f32>>) {
    match value {
        Value::String(pattern) => (Some(pattern.clone()), None),
        Value::Map(map) => {
            let pattern = map.get("pattern").and_then(|v| {
                if let Value::String(s) = v { Some(s.clone()) } else { None }
            });

            let mut options = HashMap::new();
            for (key, val) in map.iter() {
                if key != "pattern" {
                    if let Value::Number(num) = val { options.insert(key.clone(), *num); }
                }
            }

            let opts = if options.is_empty() { None } else { Some(options) };
            (pattern, opts)
        }
        _ => (None, None),
    }
}

pub fn execute_pattern(interpreter: &mut AudioInterpreter, target: &str, pattern: &str, options: Option<HashMap<String, f32>>) -> Result<()> {
    use crate::engine::audio::events::AudioEvent;

    let swing = options.as_ref().and_then(|o| o.get("swing").copied()).unwrap_or(0.0);
    let humanize = options.as_ref().and_then(|o| o.get("humanize").copied()).unwrap_or(0.0);
    let velocity_mult = options.as_ref().and_then(|o| o.get("velocity").copied()).unwrap_or(1.0);
    let tempo_override = options.as_ref().and_then(|o| o.get("tempo").copied());

    let effective_bpm = tempo_override.unwrap_or(interpreter.bpm);

    let resolved_uri = resolve_sample_uri(interpreter, target);

    let pattern_chars: Vec<char> = pattern.chars().filter(|c| !c.is_whitespace()).collect();
    let step_count = pattern_chars.len() as f32;
    if step_count == 0.0 { return Ok(()); }

    let bar_duration = (60.0 / effective_bpm) * 4.0;
    let step_duration = bar_duration / step_count;

    for (i, &ch) in pattern_chars.iter().enumerate() {
        if ch == 'x' || ch == 'X' {
            let mut time = interpreter.cursor_time + (i as f32 * step_duration);
            if swing > 0.0 && i % 2 == 1 { time += step_duration * swing; }

            #[cfg(any(feature = "cli", feature = "wasm"))]
            if humanize > 0.0 {
                use rand::Rng;
                let mut rng = rand::thread_rng();
                let offset = rng.gen_range(-humanize..humanize);
                time += offset;
            }

            let event = AudioEvent::Sample { uri: resolved_uri.clone(), start_time: time, velocity: 100.0 * velocity_mult };
            interpreter.events.events.push(event);
        }
    }

    interpreter.cursor_time += bar_duration;
    Ok(())
}

pub fn resolve_sample_uri(interpreter: &AudioInterpreter, target: &str) -> String {
    if let Some(dot_pos) = target.find('.') {
        let bank_alias = &target[..dot_pos];
        let trigger_name = &target[dot_pos + 1..];
        if let Some(Value::Map(bank_map)) = interpreter.variables.get(bank_alias) {
            if let Some(Value::String(bank_name)) = bank_map.get("_name") {
                return format!("devalang://bank/{}/{}", bank_name, trigger_name);
            }
        }
    }
    target.to_string()
}
