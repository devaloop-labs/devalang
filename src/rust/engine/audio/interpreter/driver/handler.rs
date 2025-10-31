// Full handler implementation copied from top-level module
use anyhow::Result;
use std::collections::HashMap;

use crate::language::syntax::ast::{Statement, StatementKind, Value};

use super::AudioInterpreter;

pub fn handle_let(interpreter: &mut AudioInterpreter, name: &str, value: &Value) -> Result<()> {
    // Check if this is a synth definition (has waveform parameter OR _plugin_ref)
    if let Value::Map(orig_map) = value {
        // Clone la map pour modification
        let mut map = orig_map.clone();

        // Normalize older key `synth_type` into `type` so downstream logic reads the same key.
        // Prefer chained synth_type over default "synth" placeholder when present.
        if let Some(synth_val) = map.get("synth_type").cloned() {
            let should_replace = match map.get("type") {
                Some(Value::String(s)) => {
                    let clean = s.trim_matches('"').trim_matches('\'');
                    clean == "synth" || clean.is_empty()
                }
                Some(Value::Identifier(id)) => {
                    let clean = id.trim_matches('"').trim_matches('\'');
                    clean == "synth" || clean.is_empty()
                }
                None => true,
                _ => false,
            };
            if should_replace {
                map.insert("type".to_string(), synth_val.clone());
                map.remove("synth_type");
            } else {
                // If we didn't replace, still keep synth_type (back-compat) but don't remove it
            }
        }

        // Fusionne les sous-maps de paramètres chaînés (ex: "params", "adsr", "envelope")
        let chain_keys = ["params", "adsr", "envelope"];
        for key in &chain_keys {
            if let Some(Value::Map(submap)) = map.get(*key) {
                // Buffer temporaire pour éviter le prêt mutable/immuable
                let to_insert: Vec<(String, Value)> =
                    submap.iter().map(|(k, v)| (k.clone(), v.clone())).collect();
                for (k, v) in to_insert {
                    map.insert(k, v);
                }
                map.remove(*key);
            }
        }

        // Ensure default properties are present on instantiated objects (synths/plugins)
        // so that dotted access like `mySynth.volume` resolves to a concrete Value.
        // These defaults are safe no-ops for objects that don't use them.
        map.entry("volume".to_string())
            .or_insert(Value::Number(1.0));
        map.entry("gain".to_string()).or_insert(Value::Number(1.0));
        map.entry("pan".to_string()).or_insert(Value::Number(0.0));
        map.entry("detune".to_string())
            .or_insert(Value::Number(0.0));
        // Ensure a visible type key exists for prints; prefer existing value if present
        map.entry("type".to_string())
            .or_insert(Value::String("synth".to_string()));

        // Ensure defaults for synth-like objects
        crate::utils::props::ensure_default_properties(&mut map, Some("synth"));

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
                                    let ref_parts: Vec<&str> =
                                        resolved_plugin["plugin:".len()..].split(':').collect();
                                    if ref_parts.len() == 2 {
                                        let full_plugin_name = ref_parts[0];
                                        let export_name = ref_parts[1];
                                        let plugin_parts: Vec<&str> =
                                            full_plugin_name.split('.').collect();
                                        if plugin_parts.len() == 2 {
                                            plugin_author = Some(plugin_parts[0].to_string());
                                            plugin_name = Some(plugin_parts[1].to_string());
                                            plugin_export = Some(export_name.to_string());
                                            is_plugin = true;
                                        }
                                    }
                                }
                            } else if let Some(Value::Map(export_map)) = var_map.get(prop_name) {
                                if let (
                                    Some(Value::String(author)),
                                    Some(Value::String(name)),
                                    Some(Value::String(export)),
                                ) = (
                                    export_map.get("_plugin_author"),
                                    export_map.get("_plugin_name"),
                                    export_map.get("_export_name"),
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

            let waveform = crate::engine::audio::events::extract_string(&map, "waveform", "sine");
            let attack = crate::engine::audio::events::extract_number(&map, "attack", 0.01);
            let decay = crate::engine::audio::events::extract_number(&map, "decay", 0.1);
            let sustain = crate::engine::audio::events::extract_number(&map, "sustain", 0.7);
            let release = crate::engine::audio::events::extract_number(&map, "release", 0.2);

            // Accept both String and Identifier for type (parser may emit Identifier for bare words)
            let synth_type = if let Some(v) = map.get("type") {
                match v {
                    Value::String(t) => {
                        let clean = t.trim_matches('"').trim_matches('\'');
                        if clean.is_empty() || clean == "synth" {
                            None
                        } else {
                            Some(clean.to_string())
                        }
                    }
                    Value::Identifier(id) => {
                        let clean = id.trim_matches('"').trim_matches('\'');
                        if clean.is_empty() || clean == "synth" {
                            None
                        } else {
                            Some(clean.to_string())
                        }
                    }
                    _ => None,
                }
            } else {
                None
            };

            let filters = if let Some(Value::Array(filters_arr)) = map.get("filters") {
                crate::engine::audio::events::extract_filters(filters_arr)
            } else {
                Vec::new()
            };

            // Extract LFO configuration if present
            let lfo = if let Some(Value::Map(lfo_map)) = map.get("lfo") {
                use crate::engine::audio::lfo::{LfoParams, LfoRate, LfoTarget, LfoWaveform};

                // Parse rate (Hz or tempo-synced like "1/4")
                let rate_str = if let Some(Value::Number(n)) = lfo_map.get("rate") {
                    n.to_string()
                } else if let Some(Value::String(s)) = lfo_map.get("rate") {
                    s.clone()
                } else {
                    "5.0".to_string() // Default rate
                };
                let rate = LfoRate::from_value(&rate_str);

                // Parse depth (0-1)
                let depth = if let Some(Value::Number(n)) = lfo_map.get("depth") {
                    (*n).clamp(0.0, 1.0)
                } else {
                    0.5 // Default depth
                };

                // Parse waveform (sine, triangle, square, saw)
                let waveform_str = if let Some(Value::String(s)) = lfo_map.get("shape") {
                    s.clone()
                } else if let Some(Value::String(s)) = lfo_map.get("waveform") {
                    s.clone()
                } else {
                    "sine".to_string() // Default waveform
                };
                let waveform = LfoWaveform::from_str(&waveform_str);

                // Parse target (volume, pitch, filter, pan)
                let target = if let Some(Value::String(s)) = lfo_map.get("target") {
                    LfoTarget::from_str(s).unwrap_or(LfoTarget::Volume)
                } else {
                    LfoTarget::Volume // Default target
                };

                // Parse initial phase (0-1)
                let phase = if let Some(Value::Number(n)) = lfo_map.get("phase") {
                    (*n).fract().abs() // Ensure 0-1 range
                } else {
                    0.0 // Default phase
                };

                Some(LfoParams {
                    rate,
                    depth,
                    waveform,
                    target,
                    phase,
                })
            } else {
                None
            };

            let mut options = std::collections::HashMap::new();
            let reserved_keys = if is_plugin {
                vec![
                    "attack",
                    "decay",
                    "sustain",
                    "release",
                    "type",
                    "filters",
                    "_plugin_ref",
                    "lfo",
                ]
            } else {
                vec![
                    "waveform",
                    "attack",
                    "decay",
                    "sustain",
                    "release",
                    "type",
                    "filters",
                    "_plugin_ref",
                    "lfo",
                ]
            };

            for (key, val) in map.iter() {
                if !reserved_keys.contains(&key.as_str()) {
                    match val {
                        Value::Number(n) => {
                            options.insert(key.clone(), *n);
                        }
                        Value::String(s) => {
                            if is_plugin && key == "waveform" {
                                let waveform_id = match s
                                    .trim_matches('"')
                                    .trim_matches('\'')
                                    .to_lowercase()
                                    .as_str()
                                {
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

            let final_waveform = if is_plugin {
                "plugin".to_string()
            } else {
                waveform
            };

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
                lfo,
            };

            interpreter.events.add_synth(name.to_string(), synth_def);
        }

        // Insert the normalized/augmented map into variables so dotted-property reads
        // (e.g., `mySynth.volume`) return concrete values instead of Identifier("...").
        interpreter
            .variables
            .insert(name.to_string(), Value::Map(map.clone()));
        return Ok(());
    }

    // Non-map values: fall back to storing the original value
    // If the value is a stored Trigger statement, normalize it into a Map so dotted
    // property access like `myTrigger.effects.reverb.size` works and `print myTrigger`
    // displays a clean map rather than a Statement debug dump.
    if let Value::Statement(stmt_box) = value {
        if let StatementKind::Trigger {
            entity,
            duration,
            effects,
        } = &stmt_box.kind
        {
            let mut map = HashMap::new();
            map.insert("kind".to_string(), Value::String("Trigger".to_string()));
            map.insert("entity".to_string(), Value::String(entity.clone()));
            map.insert("duration".to_string(), Value::Duration(duration.clone()));
            if let Some(eff) = effects {
                map.insert("effects".to_string(), eff.clone());
            }

            // Ensure defaults for trigger objects
            crate::utils::props::ensure_default_properties(&mut map, Some("trigger"));

            interpreter
                .variables
                .insert(name.to_string(), Value::Map(map));
            return Ok(());
        }
    }

    interpreter
        .variables
        .insert(name.to_string(), value.clone());
    Ok(())
}

pub fn handle_call(interpreter: &mut AudioInterpreter, name: &str, args: &[Value]) -> Result<()> {
    // ============================================================================
    // CALL EXECUTION (Sequential)
    // ============================================================================
    // Call accepts:
    // - Patterns (defined with `pattern name with trigger = "x---"`)
    // - Groups (defined with `group name:`)
    // ============================================================================

    // Check for user-defined function stored as a variable
    if let Some(var_val) = interpreter.variables.get(name).cloned() {
        if let Value::Statement(stmt_box) = var_val {
            if let StatementKind::Function {
                name: _fname,
                parameters,
                body,
            } = &stmt_box.kind
            {
                // Create a local variable scope for function execution
                let vars_snapshot = interpreter.variables.clone();

                // Bind parameters: resolve passed args into actual values
                for (i, param) in parameters.iter().enumerate() {
                    let bound = args.get(i).cloned().unwrap_or(Value::Null);
                    // If the bound value is an Identifier, resolve it to its actual value
                    let bound_val = match bound {
                        Value::Identifier(ref id) => {
                            interpreter.resolve_value(&Value::Identifier(id.clone()))?
                        }
                        other => other,
                    };
                    interpreter.variables.insert(param.clone(), bound_val);
                }

                // Execute the function body. Track function call depth so 'return'
                // statements are only valid within a function context.
                interpreter.function_call_depth += 1;
                let exec_result = super::collector::collect_events(interpreter, body);
                // decrement depth regardless of success or error
                interpreter.function_call_depth = interpreter.function_call_depth.saturating_sub(1);
                exec_result?;

                // Capture return value if the function executed a `return` statement.
                let mut captured_return: Option<Value> = None;
                if interpreter.returning_flag {
                    captured_return = interpreter.return_value.clone();
                    // clear the interpreter return state now that we've captured it
                    interpreter.returning_flag = false;
                    interpreter.return_value = None;
                }

                // Restore variables (local scope ends). We deliberately do not touch interpreter.events
                // so synth definitions or other global registrations performed by the function persist.
                interpreter.variables = vars_snapshot;

                // If there was a returned value, expose it to the caller scope via a special variable
                // named "__return" so callers can inspect the result. This is a simple mechanism
                // for now; higher-level expression support can be added later.
                if let Some(rv) = captured_return {
                    interpreter.variables.insert("__return".to_string(), rv);
                }

                return Ok(());
            }
            // If it's a stored pattern (inline pattern stored as Statement), handle below
            if let StatementKind::Pattern { target, .. } = &stmt_box.kind {
                if let Some(tgt) = target.as_ref() {
                    let (pattern_str, options) = interpreter.extract_pattern_data(&stmt_box.value);
                    if let Some(pat) = pattern_str {
                        interpreter.execute_pattern(tgt.as_str(), &pat, options)?;
                        return Ok(());
                    }
                }
            }
        }
    }

    // If it's a group call, execute the group body
    if let Some(body) = interpreter.groups.get(name).cloned() {
        super::collector::collect_events(interpreter, &body)?;
        return Ok(());
    }

    println!(
        "⚠️  Warning: Group, pattern or function '{}' not found",
        name
    );
    Ok(())
}

/// Execute a call as an expression and return its resulting Value.
/// This is similar to `handle_call` but returns the captured `return` value
/// from a function when present. Groups and patterns return `Value::Null`.
pub fn call_function(
    interpreter: &mut AudioInterpreter,
    name: &str,
    args: &[Value],
) -> Result<Value> {
    // If it's a stored variable that is a function statement, execute and capture return
    if let Some(var_val) = interpreter.variables.get(name).cloned() {
        if let Value::Statement(stmt_box) = var_val {
            if let StatementKind::Function {
                name: _fname,
                parameters,
                body,
            } = &stmt_box.kind
            {
                // create local variable snapshot
                let vars_snapshot = interpreter.variables.clone();

                // Bind parameters: use provided args (they are already resolved by caller)
                for (i, param) in parameters.iter().enumerate() {
                    let bound = args.get(i).cloned().unwrap_or(Value::Null);
                    // If the bound value is an Identifier, resolve it to its actual value
                    let bound_val = match bound {
                        Value::Identifier(ref id) => {
                            interpreter.resolve_value(&Value::Identifier(id.clone()))?
                        }
                        other => other,
                    };
                    interpreter.variables.insert(param.clone(), bound_val);
                }

                // Execute body in function context
                interpreter.function_call_depth += 1;

                let exec_result = super::collector::collect_events(interpreter, body);
                interpreter.function_call_depth = interpreter.function_call_depth.saturating_sub(1);
                exec_result?;

                // Capture return value if present
                let mut captured_return: Option<Value> = None;
                if interpreter.returning_flag {
                    captured_return = interpreter.return_value.clone();
                    interpreter.returning_flag = false;
                    interpreter.return_value = None;
                }

                // Restore variables (local scope ends)
                interpreter.variables = vars_snapshot;

                if let Some(rv) = captured_return {
                    return Ok(rv);
                }

                return Ok(Value::Null);
            }
            // Patterns fallthrough to below handling
            if let StatementKind::Pattern { target, .. } = &stmt_box.kind {
                if let Some(tgt) = target.as_ref() {
                    let (pattern_str, options) = interpreter.extract_pattern_data(&stmt_box.value);
                    if let Some(pat) = pattern_str {
                        interpreter.execute_pattern(tgt.as_str(), &pat, options)?;
                        return Ok(Value::Null);
                    }
                }
            }
        }
    }

    // If it's a group call, execute and return null
    if let Some(body) = interpreter.groups.get(name).cloned() {
        super::collector::collect_events(interpreter, &body)?;
        return Ok(Value::Null);
    }

    println!(
        "⚠️  Warning: Group, pattern or function '{}' not found",
        name
    );
    Ok(Value::Null)
}

pub fn execute_print(interpreter: &mut AudioInterpreter, value: &Value) -> Result<()> {
    let message = match value {
        Value::Call { name: _, args: _ } => {
            // Evaluate the call expression and convert the returned value to string
            let resolved = interpreter.resolve_value(value)?;
            interpreter.value_to_string(&resolved)
        }

        Value::String(s) => {
            if s.contains('{') && s.contains('}') {
                interpreter.interpolate_string(s)
            } else {
                s.clone()
            }
        }
        Value::Identifier(id) => {
            // Resolve identifier values (supports post-increment shorthand)
            // Support post-increment shorthand in prints: `i++` should mutate the variable.
            if id.ends_with("++") {
                let varname = id[..id.len() - 2].trim();
                // read current
                let cur = match interpreter.variables.get(varname) {
                    Some(Value::Number(n)) => *n as isize,
                    _ => 0,
                };
                // set new value
                interpreter
                    .variables
                    .insert(varname.to_string(), Value::Number((cur + 1) as f32));
                cur.to_string()
            } else {
                // Resolve the identifier using the interpreter resolver so dotted paths
                // (e.g., mySynth.volume) are properly traversed.
                let resolved = interpreter.resolve_value(&Value::Identifier(id.clone()))?;
                match resolved {
                    Value::String(s) => s.clone(),
                    Value::Number(n) => n.to_string(),
                    Value::Boolean(b) => b.to_string(),
                    Value::Array(arr) => format!("{:?}", arr),
                    Value::Map(map) => format!("{:?}", map),
                    Value::Null => format!("Identifier(\"{}\")", id),
                    other => format!("{:?}", other),
                }
            }
        }
        Value::Number(n) => n.to_string(),
        Value::Boolean(b) => b.to_string(),
        Value::Array(arr) => {
            // Treat arrays used in print as concatenation parts: resolve each part and join
            let mut parts = Vec::new();
            for v in arr.iter() {
                // If identifier with ++, handle mutation here
                if let Value::Identifier(idtok) = v {
                    if idtok.ends_with("++") {
                        let varname = idtok[..idtok.len() - 2].trim();
                        let cur = match interpreter.variables.get(varname) {
                            Some(Value::Number(n)) => *n as isize,
                            _ => 0,
                        };
                        interpreter
                            .variables
                            .insert(varname.to_string(), Value::Number((cur + 1) as f32));
                        parts.push(cur.to_string());
                        continue;
                    }
                }
                let resolved = interpreter.resolve_value(v)?;
                parts.push(interpreter.value_to_string(&resolved));
            }
            parts.join("")
        }
        Value::Map(map) => format!("{:?}", map),
        _ => format!("{:?}", value),
    };
    // Always record prints as scheduled log events so they can be replayed at playback time.
    // Record the scheduled log message (no debug-origin annotation in production)
    let log_message = message.clone();
    // Use the interpreter cursor_time as the event time.
    interpreter
        .events
        .add_log_event(log_message.clone(), interpreter.cursor_time);
    // If the interpreter is explicitly allowed to print (runtime interpreter), print immediately.
    // For offline renders (`suppress_print == true`) we do NOT print now; prints are scheduled
    // into `interpreter.events.logs` and written to a `.printlog` sidecar during the build.
    // The live playback engine replays that sidecar so prints appear in real time during playback.
    if !interpreter.suppress_print {
        // Use the global CLI logger formatting for prints so they appear as [PRINT]
        // Guard CLI-only logger behind the `cli` feature so WASM builds don't
        // reference the `crate::tools` module (which is only available for native builds).
        #[cfg(feature = "cli")]
        {
            crate::tools::logger::Logger::new().print(message.clone());
        }

        // For non-CLI builds (wasm/plugins) try forwarding to the realtime print
        // channel if provided so prints are surfaced in UIs that consume them.
        #[cfg(not(feature = "cli"))]
        {
            if let Some(tx) = &interpreter.realtime_print_tx {
                let _ = tx.send((interpreter.cursor_time, log_message.clone()));
            }
        }
    } else {
        // If printing is suppressed (offline render) but a realtime replay channel was provided,
        // forward the scheduled print to the replay thread so it can be displayed in real-time
        // while the offline render proceeds. This is a best-effort path used by some callers
        // that pipe scheduled prints directly into a playback session (when set).
        if let Some(tx) = &interpreter.realtime_print_tx {
            // forward the scheduled log message to realtime replayers (best-effort)
            let _ = tx.send((interpreter.cursor_time, log_message.clone()));
        }
    }
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
        // If handler has args, allow numeric interval to gate execution for 'beat'/'bar'
        if let Some(args) = &handler.args {
            if let Some(num_val) = args.iter().find(|v| matches!(v, Value::Number(_))) {
                if let Value::Number(n) = num_val {
                    let interval = (*n as usize).max(1);
                    if event_name == "beat" {
                        let cur = interpreter.special_vars.current_beat.floor() as usize;
                        if cur % interval != 0 {
                            continue;
                        }
                    } else if event_name == "bar" {
                        let cur = interpreter.special_vars.current_bar.floor() as usize;
                        if cur % interval != 0 {
                            continue;
                        }
                    }
                }
            }
        }

        if handler.once
            && !interpreter
                .event_registry
                .should_execute_once(event_name, index)
        {
            continue;
        }

        let body_clone = handler.body.clone();
        super::collector::collect_events(interpreter, &body_clone)?;
    }

    Ok(())
}

pub fn handle_assign(
    interpreter: &mut AudioInterpreter,
    target: &str,
    property: &str,
    value: &Value,
) -> Result<()> {
    // Support dotted targets like "myTrigger.effects.reverb" as the target string.
    // The parser may pass the full path as `target` and the last path segment as `property`.
    if target.contains('.') {
        let parts: Vec<&str> = target.split('.').collect();
        let root = parts[0];
        if let Some(root_val) = interpreter.variables.get_mut(root) {
            // Traverse into nested maps to reach the parent map where to insert `property`.
            let mut current = root_val;
            for seg in parts.iter().skip(1) {
                match current {
                    Value::Map(map) => {
                        if !map.contains_key(*seg) {
                            // create nested map if missing
                            map.insert((*seg).to_string(), Value::Map(HashMap::new()));
                        }
                        current = map.get_mut(*seg).unwrap();
                    }
                    _ => {
                        return Err(anyhow::anyhow!(
                            "Cannot traverse into non-map segment '{}' when assigning to '{}'",
                            seg,
                            target
                        ));
                    }
                }
            }

            // Now `current` should be a Value::Map where we insert `property`.
            if let Value::Map(map) = current {
                map.insert(property.to_string(), value.clone());

                // If the root object is a synth, update its synth definition
                if interpreter.events.synths.contains_key(root) {
                    if let Some(Value::Map(root_map)) = interpreter.variables.get(root) {
                        let map_clone = root_map.clone();
                        let updated_def = interpreter.extract_synth_def_from_map(&map_clone)?;
                        interpreter
                            .events
                            .synths
                            .insert(root.to_string(), updated_def);
                    }
                }
            } else {
                return Err(anyhow::anyhow!(
                    "Cannot assign property '{}' to non-map target '{}'",
                    property,
                    target
                ));
            }
        } else {
            return Err(anyhow::anyhow!("Variable '{}' not found", root));
        }
    } else {
        if let Some(var) = interpreter.variables.get_mut(target) {
            if let Value::Map(map) = var {
                map.insert(property.to_string(), value.clone());

                if interpreter.events.synths.contains_key(target) {
                    let map_clone = map.clone();
                    let updated_def = interpreter.extract_synth_def_from_map(&map_clone)?;
                    interpreter
                        .events
                        .synths
                        .insert(target.to_string(), updated_def);
                }
            } else {
                return Err(anyhow::anyhow!(
                    "Cannot assign property '{}' to non-map variable '{}'",
                    property,
                    target
                ));
            }
        } else {
            return Err(anyhow::anyhow!("Variable '{}' not found", target));
        }
    }

    Ok(())
}

pub fn extract_synth_def_from_map(
    _interpreter: &AudioInterpreter,
    map: &HashMap<String, Value>,
) -> Result<crate::engine::audio::events::SynthDefinition> {
    use crate::engine::audio::events::extract_filters;
    use crate::engine::audio::lfo::{LfoParams, LfoRate, LfoTarget, LfoWaveform};

    let waveform = crate::engine::audio::events::extract_string(map, "waveform", "sine");
    let attack = crate::engine::audio::events::extract_number(map, "attack", 0.01);
    let decay = crate::engine::audio::events::extract_number(map, "decay", 0.1);
    let sustain = crate::engine::audio::events::extract_number(map, "sustain", 0.7);
    let release = crate::engine::audio::events::extract_number(map, "release", 0.2);

    // Accept both String and Identifier for type (and synth_type alias)
    let synth_type = if let Some(v) = map.get("type") {
        match v {
            Value::String(t) => {
                let clean = t.trim_matches('"').trim_matches('\'');
                if clean.is_empty() || clean == "synth" {
                    None
                } else {
                    Some(clean.to_string())
                }
            }
            Value::Identifier(id) => {
                let clean = id.trim_matches('"').trim_matches('\'');
                if clean.is_empty() || clean == "synth" {
                    None
                } else {
                    Some(clean.to_string())
                }
            }
            _ => None,
        }
    } else if let Some(v2) = map.get("synth_type") {
        match v2 {
            Value::String(t2) => {
                let clean = t2.trim_matches('"').trim_matches('\'');
                if clean.is_empty() || clean == "synth" {
                    None
                } else {
                    Some(clean.to_string())
                }
            }
            Value::Identifier(id2) => {
                let clean = id2.trim_matches('"').trim_matches('\'');
                if clean.is_empty() || clean == "synth" {
                    None
                } else {
                    Some(clean.to_string())
                }
            }
            _ => None,
        }
    } else {
        None
    };

    let filters = if let Some(Value::Array(filters_arr)) = map.get("filters") {
        extract_filters(filters_arr)
    } else {
        Vec::new()
    };

    let plugin_author = if let Some(Value::String(s)) = map.get("plugin_author") {
        Some(s.clone())
    } else {
        None
    };
    let plugin_name = if let Some(Value::String(s)) = map.get("plugin_name") {
        Some(s.clone())
    } else {
        None
    };
    let plugin_export = if let Some(Value::String(s)) = map.get("plugin_export") {
        Some(s.clone())
    } else {
        None
    };

    // Extract LFO configuration if present
    let lfo = if let Some(Value::Map(lfo_map)) = map.get("lfo") {
        // Parse rate (Hz or tempo-synced like "1/4")
        let rate_str = if let Some(Value::Number(n)) = lfo_map.get("rate") {
            n.to_string()
        } else if let Some(Value::String(s)) = lfo_map.get("rate") {
            s.clone()
        } else {
            "5.0".to_string() // Default rate
        };
        let rate = LfoRate::from_value(&rate_str);

        // Parse depth (0-1)
        let depth = if let Some(Value::Number(n)) = lfo_map.get("depth") {
            (*n).clamp(0.0, 1.0)
        } else {
            0.5 // Default depth
        };

        // Parse waveform (sine, triangle, square, saw)
        let waveform_str = if let Some(Value::String(s)) = lfo_map.get("shape") {
            s.clone()
        } else if let Some(Value::String(s)) = lfo_map.get("waveform") {
            s.clone()
        } else {
            "sine".to_string() // Default waveform
        };
        let waveform = LfoWaveform::from_str(&waveform_str);

        // Parse target (volume, pitch, filter, pan)
        let target = if let Some(Value::String(s)) = lfo_map.get("target") {
            LfoTarget::from_str(s).unwrap_or(LfoTarget::Volume)
        } else {
            LfoTarget::Volume // Default target
        };

        // Parse initial phase (0-1)
        let phase = if let Some(Value::Number(n)) = lfo_map.get("phase") {
            (*n).fract().abs() // Ensure 0-1 range
        } else {
            0.0 // Default phase
        };

        Some(LfoParams {
            rate,
            depth,
            waveform,
            target,
            phase,
        })
    } else {
        None
    };

    let mut options = HashMap::new();
    for (key, val) in map.iter() {
        if ![
            "waveform",
            "attack",
            "decay",
            "sustain",
            "release",
            "type",
            "filters",
            "plugin_author",
            "plugin_name",
            "plugin_export",
            "lfo",
        ]
        .contains(&key.as_str())
        {
            if let Value::Number(n) = val {
                options.insert(key.clone(), *n);
            } else if let Value::String(s) = val {
                if key == "waveform" || key.starts_with("_") {
                    continue;
                }
                if let Ok(n) = s.parse::<f32>() {
                    options.insert(key.clone(), n);
                }
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
        lfo,
    })
}

pub fn handle_load(interpreter: &mut AudioInterpreter, source: &str, alias: &str) -> Result<()> {
    use std::path::Path;

    let path = Path::new(source);
    // Determine extension
    if let Some(ext) = path
        .extension()
        .and_then(|s| s.to_str())
        .map(|s| s.to_lowercase())
    {
        match ext.as_str() {
            "mid" | "midi" => {
                use crate::engine::audio::midi::load_midi_file;
                let midi_data = load_midi_file(path)?;
                interpreter.variables.insert(alias.to_string(), midi_data);
                // MIDI file loaded (silent)
                Ok(())
            }
            "wav" | "flac" | "mp3" | "ogg" => {
                // For native/CLI builds, register sample via the samples subsystem.
                #[cfg(feature = "cli")]
                {
                    use crate::engine::audio::samples;
                    let registered = samples::register_sample_from_path(path)?;
                    // Record the sample URI under the alias variable as a string (consistent with triggers)
                    interpreter
                        .variables
                        .insert(alias.to_string(), Value::String(registered.clone()));
                    // Sample file loaded (silent)
                    return Ok(());
                }

                // For non-CLI builds (WASM/plugins), fallback to storing the original path as a string.
                #[cfg(not(feature = "cli"))]
                {
                    interpreter
                        .variables
                        .insert(alias.to_string(), Value::String(source.to_string()));
                    return Ok(());
                }
            }
            _ => Err(anyhow::anyhow!("Unsupported file type for @load: {}", ext)),
        }
    } else {
        Err(anyhow::anyhow!(
            "Cannot determine file extension for {}",
            source
        ))
    }
}

pub fn handle_bind(
    interpreter: &mut AudioInterpreter,
    source: &str,
    target: &str,
    options: &Value,
) -> Result<()> {
    use std::collections::HashMap as StdHashMap;

    // Support bindings that reference runtime MIDI device mappings like:
    //   bind myKickPattern -> mapping.out.myDeviceA with { port: 1, channel: 10 }
    //   bind mapping.in.myDeviceB with { port: 2, channel: 10 } -> mySynth
    // When a 'mapping.*' path is used, register a lightweight mapping entry in the
    // interpreter variables and expose convenience variables:
    //   mapping.<in|out>.<device>.<noteOn|noteOff|rest>
    if source.starts_with("mapping.") || target.starts_with("mapping.") {
        // extract options if present
        let opts_map: StdHashMap<String, Value> = if let Value::Map(m) = options {
            m.clone()
        } else {
            StdHashMap::new()
        };

        // Helper function to create mapping variables and bookkeeping
        fn create_and_insert(
            path: &str,
            opts_map: &StdHashMap<String, Value>,
            interpreter: &mut AudioInterpreter,
        ) -> Option<(String, String)> {
            let parts: Vec<&str> = path.split('.').collect();
            if parts.len() >= 3 {
                let direction = parts[1]; // "in" or "out"
                let device = parts[2];

                let mut map = StdHashMap::new();
                map.insert(
                    "_type".to_string(),
                    Value::String("midi_mapping".to_string()),
                );
                map.insert(
                    "direction".to_string(),
                    Value::String(direction.to_string()),
                );
                map.insert("device".to_string(), Value::String(device.to_string()));

                // merge provided options
                for (k, v) in opts_map.iter() {
                    map.insert(k.clone(), v.clone());
                }

                // Ensure mapping defaults
                crate::utils::props::ensure_default_properties(&mut map, Some("mapping"));

                interpreter
                    .variables
                    .insert(path.to_string(), Value::Map(map.clone()));

                // Expose event variables for convenience
                let note_on = format!("mapping.{}.{}.noteOn", direction, device);
                let note_off = format!("mapping.{}.{}.noteOff", direction, device);
                let rest = format!("mapping.{}.{}.rest", direction, device);
                interpreter
                    .variables
                    .insert(note_on.clone(), Value::String(note_on.clone()));
                interpreter
                    .variables
                    .insert(note_off.clone(), Value::String(note_off.clone()));
                interpreter
                    .variables
                    .insert(rest.clone(), Value::String(rest.clone()));

                return Some((direction.to_string(), device.to_string()));
            }
            None
        }

        // If source is mapping.* (incoming mapping binds to target instrument)
        if source.starts_with("mapping.") {
            if let Some((direction, device)) = create_and_insert(source, &opts_map, interpreter) {
                // Record association when target is not also mapping.*
                if !target.starts_with("mapping.") {
                    let mut bmap = StdHashMap::new();
                    bmap.insert("instrument".to_string(), Value::String(target.to_string()));
                    bmap.insert("direction".to_string(), Value::String(direction.clone()));
                    bmap.insert("device".to_string(), Value::String(device.clone()));
                    for (k, v) in opts_map.iter() {
                        bmap.insert(k.clone(), v.clone());
                    }
                    interpreter
                        .variables
                        .insert(format!("__mapping_bind::{}", source), Value::Map(bmap));

                    // If we have a midi_manager, try to open the input port (for incoming mappings)
                    #[cfg(feature = "cli")]
                    if let Some(manager) = &mut interpreter.midi_manager {
                        if let Some(Value::Number(port_num)) = opts_map.get("port") {
                            let idx = *port_num as usize;
                            if let Ok(mut mgr) = manager.lock() {
                                // use device as name for identification
                                let _ = mgr.open_input_by_index(idx, &device);
                            }
                        }
                    }
                }
            }
        }

        // If target is mapping.* (binding a sequence/instrument to an external device)
        if target.starts_with("mapping.") {
            if let Some((direction, device)) = create_and_insert(target, &opts_map, interpreter) {
                if !source.starts_with("mapping.") {
                    let mut bmap = StdHashMap::new();
                    bmap.insert("source".to_string(), Value::String(source.to_string()));
                    bmap.insert("direction".to_string(), Value::String(direction.clone()));
                    bmap.insert("device".to_string(), Value::String(device.clone()));
                    for (k, v) in opts_map.iter() {
                        bmap.insert(k.clone(), v.clone());
                    }
                    interpreter
                        .variables
                        .insert(format!("__mapping_bind::{}", target), Value::Map(bmap));

                    // If we have a midi_manager, try to open the output port (for outgoing mappings)
                    #[cfg(feature = "cli")]
                    if let Some(manager) = &mut interpreter.midi_manager {
                        if let Some(Value::Number(port_num)) = opts_map.get("port") {
                            let idx = *port_num as usize;
                            if let Ok(mut mgr) = manager.lock() {
                                let _ = mgr.open_output_by_name(&device, idx);
                            }
                        }
                    }
                }
            }
        }

        // Nothing more to schedule here at audio event level; actual MIDI I/O handlers
        // will be responsible for reacting to incoming messages and emitting events into
        // the interpreter event registry, and for flushing outgoing bound sequences to
        // MIDI device ports when appropriate.
        return Ok(());
    }

    // Fallback: existing behaviour (binding MIDI file data to a synth)
    let midi_data = interpreter
        .variables
        .get(source)
        .ok_or_else(|| anyhow::anyhow!("MIDI source '{}' not found", source))?
        .clone();

    if let Value::Map(midi_map) = &midi_data {
        let notes = midi_map
            .get("notes")
            .ok_or_else(|| anyhow::anyhow!("MIDI data has no notes"))?;

        if let Value::Array(notes_array) = notes {
            let _synth_def = interpreter
                .events
                .synths
                .get(target)
                .ok_or_else(|| anyhow::anyhow!("Synth '{}' not found", target))?
                .clone();

            let default_velocity = 100;
            let mut velocity = default_velocity;

            if let Value::Map(opts) = options {
                if let Some(Value::Number(v)) = opts.get("velocity") {
                    velocity = *v as u8;
                }
            }

            // Determine MIDI file BPM (if present) so we can rescale times to interpreter BPM
            // Default to interpreter.bpm when the MIDI file has no BPM metadata
            let midi_bpm =
                crate::engine::audio::events::extract_number(midi_map, "bpm", interpreter.bpm);

            for note_val in notes_array {
                if let Value::Map(note_map) = note_val {
                    let time = crate::engine::audio::events::extract_number(note_map, "time", 0.0);
                    let note =
                        crate::engine::audio::events::extract_number(note_map, "note", 60.0) as u8;
                    let note_velocity = crate::engine::audio::events::extract_number(
                        note_map,
                        "velocity",
                        velocity as f32,
                    ) as u8;
                    // Duration may be present (ms) from MIDI loader; fallback to 500 ms
                    let duration_ms =
                        crate::engine::audio::events::extract_number(note_map, "duration", 500.0);

                    use crate::engine::audio::events::AudioEvent;
                    let synth_def = interpreter
                        .events
                        .get_synth(target)
                        .cloned()
                        .unwrap_or_default();
                    // Rescale times according to interpreter BPM vs MIDI file BPM.
                    // If midi_bpm == interpreter.bpm this is a no-op. We compute factor = midi_bpm / interpreter.bpm
                    let interp_bpm = interpreter.bpm;
                    let factor = if interp_bpm > 0.0 {
                        midi_bpm / interp_bpm
                    } else {
                        1.0
                    };

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
                        effects: None,
                        use_per_note_automation: false,
                    };

                    // bound note scheduled
                    interpreter.events.events.push(event);
                }
            }

            // Bound notes from source to target
        }
    }

    Ok(())
}

#[cfg(feature = "cli")]
pub fn handle_use_plugin(
    interpreter: &mut AudioInterpreter,
    author: &str,
    name: &str,
    alias: &str,
) -> Result<()> {
    use crate::engine::plugin::loader::load_plugin;

    match load_plugin(author, name) {
        Ok((info, _wasm_bytes)) => {
            let mut plugin_map = HashMap::new();
            plugin_map.insert("_type".to_string(), Value::String("plugin".to_string()));
            plugin_map.insert("_author".to_string(), Value::String(info.author.clone()));
            plugin_map.insert("_name".to_string(), Value::String(info.name.clone()));

            if let Some(version) = &info.version {
                plugin_map.insert("_version".to_string(), Value::String(version.clone()));
            }

            for export in &info.exports {
                let mut export_map = HashMap::new();
                export_map.insert(
                    "_plugin_author".to_string(),
                    Value::String(info.author.clone()),
                );
                export_map.insert("_plugin_name".to_string(), Value::String(info.name.clone()));
                export_map.insert(
                    "_export_name".to_string(),
                    Value::String(export.name.clone()),
                );
                export_map.insert(
                    "_export_kind".to_string(),
                    Value::String(export.kind.clone()),
                );

                plugin_map.insert(export.name.clone(), Value::Map(export_map));
            }
            // Ensure plugin map has default properties available for dotted access
            crate::utils::props::ensure_default_properties(&mut plugin_map, Some("plugin"));

            interpreter
                .variables
                .insert(alias.to_string(), Value::Map(plugin_map));
        }
        Err(e) => {
            eprintln!("❌ Failed to load plugin {}.{}: {}", author, name, e);
            return Err(anyhow::anyhow!("Failed to load plugin: {}", e));
        }
    }

    Ok(())
}

#[cfg(not(feature = "cli"))]
pub fn handle_use_plugin(
    interpreter: &mut AudioInterpreter,
    author: &str,
    name: &str,
    alias: &str,
) -> Result<()> {
    // Plugin loading not supported in this build (WASM/plugin builds). Insert a minimal placeholder so scripts can still reference the alias.
    let mut plugin_map = HashMap::new();
    plugin_map.insert(
        "_type".to_string(),
        Value::String("plugin_stub".to_string()),
    );
    plugin_map.insert("_author".to_string(), Value::String(author.to_string()));
    plugin_map.insert("_name".to_string(), Value::String(name.to_string()));
    // Ensure stubs also expose defaults for dotted access
    crate::utils::props::ensure_default_properties(&mut plugin_map, Some("plugin"));

    interpreter
        .variables
        .insert(alias.to_string(), Value::Map(plugin_map));
    Ok(())
}

pub fn handle_bank(
    interpreter: &mut AudioInterpreter,
    name: &str,
    alias: &Option<String>,
) -> Result<()> {
    let target_alias = alias
        .clone()
        .unwrap_or_else(|| name.split('.').last().unwrap_or(name).to_string());

    if let Some(existing_value) = interpreter.variables.get(name) {
        interpreter
            .variables
            .insert(target_alias.clone(), existing_value.clone());
    } else {
        #[cfg(feature = "wasm")]
        {
            use crate::web::registry::banks::REGISTERED_BANKS;
            REGISTERED_BANKS.with(|banks| {
                for bank in banks.borrow().iter() {
                    if bank.full_name == *name {
                        if let Some(Value::Map(bank_map)) = interpreter.variables.get(&bank.alias) {
                            interpreter
                                .variables
                                .insert(target_alias.clone(), Value::Map(bank_map.clone()));
                        }
                    }
                }
            });
        }

        #[cfg(not(feature = "wasm"))]
        {
            if let Ok(current_dir) = std::env::current_dir() {
                match interpreter.banks.register_bank(
                    target_alias.clone(),
                    &name,
                    &current_dir,
                    &current_dir,
                ) {
                    Ok(_) => {
                        let mut bank_map = HashMap::new();
                        bank_map.insert("_name".to_string(), Value::String(name.to_string()));
                        bank_map.insert("_alias".to_string(), Value::String(target_alias.clone()));
                        interpreter
                            .variables
                            .insert(target_alias.clone(), Value::Map(bank_map));
                    }
                    Err(e) => {
                        eprintln!("⚠️ Failed to register bank '{}': {}", name, e);
                        let mut bank_map = HashMap::new();
                        bank_map.insert("_name".to_string(), Value::String(name.to_string()));
                        bank_map.insert("_alias".to_string(), Value::String(target_alias.clone()));
                        interpreter
                            .variables
                            .insert(target_alias.clone(), Value::Map(bank_map));
                    }
                }
            } else {
                let mut bank_map = HashMap::new();
                bank_map.insert("_name".to_string(), Value::String(name.to_string()));
                bank_map.insert("_alias".to_string(), Value::String(target_alias.clone()));
                interpreter
                    .variables
                    .insert(target_alias.clone(), Value::Map(bank_map));
                eprintln!(
                    "⚠️ Could not determine cwd to register bank '{}', registered minimal alias.",
                    name
                );
            }
        }
    }

    Ok(())
}

pub fn handle_trigger(
    interpreter: &mut AudioInterpreter,
    entity: &str,
    effects: Option<&crate::language::syntax::ast::Value>,
) -> Result<()> {
    let resolved_entity = if entity.starts_with('.') {
        &entity[1..]
    } else {
        entity
    };

    // If this resolved entity refers to a variable that contains a Trigger statement,
    // execute that stored trigger instead (supports: let t = .bank.kick -> reverse(true); .t )
    if let Some(var_val) = interpreter.variables.get(resolved_entity).cloned() {
        if let crate::language::syntax::ast::Value::Statement(stmt_box) = var_val {
            if let crate::language::syntax::ast::StatementKind::Trigger {
                entity: inner_entity,
                duration: _,
                effects: stored_effects,
            } = &stmt_box.kind
            {
                // Avoid direct recursion if someone stored a trigger that points to itself
                if inner_entity != resolved_entity {
                    // Prefer stored effects when present, otherwise fall back to effects passed in
                    let chosen_effects = stored_effects.as_ref().or(effects);
                    return handle_trigger(interpreter, inner_entity, chosen_effects);
                } else {
                    return Ok(());
                }
            }
        }
    }

    if resolved_entity.contains('.') {
        let parts: Vec<&str> = resolved_entity.split('.').collect();
        if parts.len() == 2 {
            let (var_name, property) = (parts[0], parts[1]);

            if let Some(Value::Map(map)) = interpreter.variables.get(var_name) {
                if let Some(Value::String(sample_uri)) = map.get(property) {
                    let uri = sample_uri.trim_matches('"').trim_matches('\'');
                    // scheduling sample at current cursor_time
                    interpreter.events.add_sample_event_with_effects(
                        uri,
                        interpreter.cursor_time,
                        1.0,
                        effects.cloned(),
                    );
                    let beat_duration = interpreter.beat_duration();
                    interpreter.cursor_time += beat_duration;
                } else {
                    #[cfg(not(feature = "wasm"))]
                    {
                        // First try to produce an internal devalang://bank URI (preferred, supports lazy loading)
                        let resolved_uri = interpreter.resolve_sample_uri(resolved_entity);
                        if resolved_uri != resolved_entity {
                            // scheduling resolved sample
                            interpreter.events.add_sample_event_with_effects(
                                &resolved_uri,
                                interpreter.cursor_time,
                                1.0,
                                effects.cloned(),
                            );
                            let beat_duration = interpreter.beat_duration();
                            interpreter.cursor_time += beat_duration;
                        } else if let Some(pathbuf) =
                            interpreter.banks.resolve_trigger(var_name, property)
                        {
                            if let Some(path_str) = pathbuf.to_str() {
                                // scheduling sample via bank path
                                interpreter.events.add_sample_event_with_effects(
                                    path_str,
                                    interpreter.cursor_time,
                                    1.0,
                                    effects.cloned(),
                                );
                                let beat_duration = interpreter.beat_duration();
                                interpreter.cursor_time += beat_duration;
                            } else {
                                println!(
                                    "⚠️ Resolution failed for {}.{} (invalid path)",
                                    var_name, property
                                );
                            }
                        } else {
                            // no path found in BankRegistry
                        }
                    }
                }
            }
        }
    } else {
        if let Some(Value::String(sample_uri)) = interpreter.variables.get(resolved_entity) {
            let uri = sample_uri.trim_matches('"').trim_matches('\'');
            interpreter.events.add_sample_event_with_effects(
                uri,
                interpreter.cursor_time,
                1.0,
                effects.cloned(),
            );
            let beat_duration = interpreter.beat_duration();
            interpreter.cursor_time += beat_duration;
        }
    }

    // Note: do not call interpreter.render_audio() here - rendering is handled by the build pipeline.
    // Trigger queued for rendering (events collected)

    Ok(())
}

pub fn extract_pattern_data(
    _interpreter: &AudioInterpreter,
    value: &Value,
) -> (Option<String>, Option<HashMap<String, f32>>) {
    match value {
        Value::String(pattern) => (Some(pattern.clone()), None),
        Value::Map(map) => {
            let pattern = map.get("pattern").and_then(|v| {
                if let Value::String(s) = v {
                    Some(s.clone())
                } else {
                    None
                }
            });

            let mut options = HashMap::new();
            for (key, val) in map.iter() {
                if key != "pattern" {
                    if let Value::Number(num) = val {
                        options.insert(key.clone(), *num);
                    }
                }
            }

            let opts = if options.is_empty() {
                None
            } else {
                Some(options)
            };
            (pattern, opts)
        }
        _ => (None, None),
    }
}

pub fn execute_pattern(
    interpreter: &mut AudioInterpreter,
    target: &str,
    pattern: &str,
    options: Option<HashMap<String, f32>>,
) -> Result<()> {
    use crate::engine::audio::events::AudioEvent;

    let swing = options
        .as_ref()
        .and_then(|o| o.get("swing").copied())
        .unwrap_or(0.0);
    let humanize = options
        .as_ref()
        .and_then(|o| o.get("humanize").copied())
        .unwrap_or(0.0);
    let velocity_mult = options
        .as_ref()
        .and_then(|o| o.get("velocity").copied())
        .unwrap_or(1.0);
    let tempo_override = options.as_ref().and_then(|o| o.get("tempo").copied());

    let effective_bpm = tempo_override.unwrap_or(interpreter.bpm);

    let resolved_uri = resolve_sample_uri(interpreter, target);

    let pattern_chars: Vec<char> = pattern.chars().filter(|c| !c.is_whitespace()).collect();
    let step_count = pattern_chars.len() as f32;
    if step_count == 0.0 {
        return Ok(());
    }

    let bar_duration = (60.0 / effective_bpm) * 4.0;
    let step_duration = bar_duration / step_count;

    for (i, &ch) in pattern_chars.iter().enumerate() {
        if ch == 'x' || ch == 'X' {
            let mut time = interpreter.cursor_time + (i as f32 * step_duration);
            if swing > 0.0 && i % 2 == 1 {
                time += step_duration * swing;
            }

            #[cfg(any(feature = "cli", feature = "wasm"))]
            if humanize > 0.0 {
                use rand::Rng;
                let mut rng = rand::thread_rng();
                let offset = rng.gen_range(-humanize..humanize);
                time += offset;
            }

            let event = AudioEvent::Sample {
                uri: resolved_uri.clone(),
                start_time: time,
                velocity: velocity_mult, // Already in 0-1 range, not MIDI 0-127
                effects: None,
            };
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
