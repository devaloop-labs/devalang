use crate::engine::audio::effects::processors::{
    DelayProcessor, DriveProcessor, EffectProcessor, ReverbProcessor,
};
use crate::engine::audio::events::{
    AudioEventList, SynthDefinition, extract_filters, extract_number, extract_string,
};
use crate::engine::audio::generator::{
    SynthParams, generate_chord_with_options, generate_note_with_options,
};
use crate::engine::events::{EventHandler, EventRegistry};
use crate::engine::functions::FunctionRegistry;
use crate::engine::functions::note::parse_note_to_midi;
use crate::engine::special_vars::{SpecialVarContext, is_special_var, resolve_special_var};
use crate::language::syntax::ast::{Statement, StatementKind, Value};
/// Audio interpreter driver - main execution loop
use anyhow::Result;
use rayon::prelude::*;
use std::collections::HashMap;

pub struct AudioInterpreter {
    pub sample_rate: u32,
    pub bpm: f32,
    function_registry: FunctionRegistry,
    events: AudioEventList,
    pub(crate) variables: HashMap<String, Value>,
    groups: HashMap<String, Vec<Statement>>,
    cursor_time: f32,
    pub special_vars: SpecialVarContext,
    pub event_registry: EventRegistry,
    /// Track current statement location for better error reporting
    current_statement_location: Option<(usize, usize)>, // (line, column)
}

impl AudioInterpreter {
    pub fn new(sample_rate: u32) -> Self {
        Self {
            sample_rate,
            bpm: 120.0,
            function_registry: FunctionRegistry::new(),
            events: AudioEventList::new(),
            variables: HashMap::new(),
            groups: HashMap::new(),
            cursor_time: 0.0,
            special_vars: SpecialVarContext::new(120.0, sample_rate),
            event_registry: EventRegistry::new(),
            current_statement_location: None,
        }
    }

    pub fn interpret(&mut self, statements: &[Statement]) -> Result<Vec<f32>> {
        // Initialize special vars context
        let total_duration = self.calculate_total_duration(statements)?;
        self.special_vars.total_duration = total_duration;
        self.special_vars.update_bpm(self.bpm);

        // Phase 1: Collect events
        self.collect_events(statements)?;

        // Phase 2: Render audio
        self.render_audio()
    }

    /// Get reference to collected audio events (for MIDI export)
    pub fn events(&self) -> &AudioEventList {
        &self.events
    }

    /// Get current statement location for error reporting
    pub fn current_statement_location(&self) -> Option<(usize, usize)> {
        self.current_statement_location
    }

    /// Calculate approximate total duration by scanning statements
    fn calculate_total_duration(&self, _statements: &[Statement]) -> Result<f32> {
        // For now, return a default duration (will be updated during collect_events)
        Ok(60.0) // Default 60 seconds
    }

    pub(crate) fn collect_events(&mut self, statements: &[Statement]) -> Result<()> {
        // Partition statements: separate spawns from others for parallel execution
        let (spawns, others): (Vec<_>, Vec<_>) = statements
            .iter()
            .partition(|stmt| matches!(stmt.kind, StatementKind::Spawn { .. }));

        // Execute sequential statements first
        for stmt in &others {
            // Track current statement location for error reporting
            self.current_statement_location = Some((stmt.line, stmt.column));

            // Update special variables context with current time
            self.special_vars.update_time(self.cursor_time);

            match &stmt.kind {
                StatementKind::Let { name, value } => {
                    if let Some(val) = value {
                        self.handle_let(name, val)?;
                    }
                }

                StatementKind::ArrowCall {
                    target,
                    method,
                    args,
                } => {
                    // Extract chain if present from stmt.value
                    let chain = if let Value::Map(map) = &stmt.value {
                        map.get("chain").and_then(|v| {
                            if let Value::Array(arr) = v {
                                Some(arr.as_slice())
                            } else {
                                None
                            }
                        })
                    } else {
                        None
                    };

                    // Execute arrow call with error capture for WASM
                    let context = super::statements::arrow_call::execute_arrow_call(
                        &self.function_registry,
                        target,
                        method,
                        args,
                        chain,
                        self.cursor_time,
                        self.bpm,
                    )
                    .map_err(|e| {
                        // Capture error with statement location for WASM debugging
                        #[cfg(target_arch = "wasm32")]
                        {
                            use crate::web::registry::debug;
                            if debug::is_debug_errors_enabled() {
                                debug::push_parse_error_from_parts(
                                    format!("{}", e),
                                    stmt.line,
                                    stmt.column,
                                    "RuntimeError".to_string(),
                                );
                            }
                        }
                        e
                    })?;

                    // Extract event from context and add to events list
                    self.extract_audio_event(target, &context)?;

                    // Update cursor time
                    self.cursor_time += context.duration;
                }

                StatementKind::Tempo => {
                    if let Value::Number(bpm_value) = &stmt.value {
                        self.set_bpm(*bpm_value);
                    }
                }

                StatementKind::Sleep => {
                    // Extract duration from stmt.value
                    if let Value::Number(duration) = &stmt.value {
                        self.cursor_time += duration / 1000.0; // Convert ms to seconds
                    }
                }

                StatementKind::Group { name, body } => {
                    // Store group definition for later execution
                    self.groups.insert(name.clone(), body.clone());
                }

                StatementKind::Call { name, args: _ } => {
                    // Check if this is an inline pattern assignment
                    if let Value::Map(map) = &stmt.value {
                        if let Some(Value::Boolean(true)) = map.get("inline_pattern") {
                            // Execute inline pattern
                            if let (Some(Value::String(target)), Some(Value::String(pattern))) = 
                                (map.get("target"), map.get("pattern")) 
                            {
                                println!("🎵 Call inline pattern: {} = \"{}\"", target, pattern);
                                let target = target.clone();
                                let pattern = pattern.clone();
                                self.execute_pattern(&target, &pattern, None)?;
                                continue;
                            }
                        }
                    }

                    // Check if this is a pattern call (not a group)
                    // Clone the pattern data first to avoid borrow issues
                    let pattern_data: Option<(String, String, String, Option<HashMap<String, f32>>)> = 
                        if let Some(pattern_value) = self.variables.get(name) {
                            if let Value::Statement(stmt_box) = pattern_value {
                                if let StatementKind::Pattern { target, .. } = &stmt_box.kind {
                                    if let Some(tgt) = target {
                                        let (pattern_str, options) = self.extract_pattern_data(&stmt_box.value);
                                        if let Some(pat) = pattern_str {
                                            Some((name.clone(), tgt.clone(), pat, options))
                                        } else {
                                            None
                                        }
                                    } else {
                                        None
                                    }
                                } else {
                                    None
                                }
                            } else {
                                None
                            }
                        } else {
                            None
                        };

                    if let Some((pname, target, pattern, options)) = pattern_data {
                        println!("🎵 Call pattern: {} with {}", pname, target);
                        self.execute_pattern(&target, &pattern, options)?;
                        continue;
                    }

                    // Execute group by name
                    if let Some(body) = self.groups.get(name).cloned() {
                        println!("📞 Call group: {}", name);
                        self.collect_events(&body)?;
                    } else {
                        println!("⚠️  Warning: Group or pattern '{}' not found", name);
                    }
                }

                StatementKind::Spawn { .. } => {
                    // Spawns are handled separately in parallel execution below
                    // This branch should not be reached due to partitioning
                    unreachable!("Spawn statements should be handled in parallel section");
                }

                StatementKind::Loop { count, body } => {
                    // Execute loop N times
                    self.execute_loop(count, body)?;
                }

                StatementKind::For {
                    variable,
                    iterable,
                    body,
                } => {
                    // Execute for each item in array/range
                    self.execute_for(variable, iterable, body)?;
                }

                StatementKind::Print => {
                    // Execute print with variable interpolation
                    self.execute_print(&stmt.value)?;
                }

                StatementKind::If {
                    condition,
                    body,
                    else_body,
                } => {
                    // Execute condition and run appropriate branch
                    self.execute_if(condition, body, else_body)?;
                }

                StatementKind::On { event, args, body } => {
                    // Register event handler
                    let once = args
                        .as_ref()
                        .and_then(|a| a.first())
                        .and_then(|v| {
                            if let Value::String(s) = v {
                                Some(s == "once")
                            } else {
                                None
                            }
                        })
                        .unwrap_or(false);

                    let handler = EventHandler {
                        event_name: event.clone(),
                        body: body.clone(),
                        once,
                    };

                    self.event_registry.register_handler(handler);
                    println!("📡 Event handler registered: {} (once={})", event, once);
                }

                StatementKind::Emit { event, payload } => {
                    // Emit event with payload
                    let data = if let Some(Value::Map(map)) = payload {
                        map.clone()
                    } else {
                        HashMap::new()
                    };

                    self.event_registry
                        .emit(event.clone(), data, self.cursor_time);

                    // Execute matching handlers
                    self.execute_event_handlers(event)?;

                    println!("📤 Event emitted: {}", event);
                }

                StatementKind::Assign { target, property } => {
                    // Handle property assignment: target.property = value
                    self.handle_assign(target, property, &stmt.value)?;
                }

                StatementKind::Load { source, alias } => {
                    // Load MIDI file and store in variables
                    self.handle_load(source, alias)?;
                }

                StatementKind::Pattern { name, target } => {
                    // Store pattern definition in variables
                    // Pattern will be executed when called
                    let pattern_stmt = Statement {
                        kind: StatementKind::Pattern {
                            name: name.clone(),
                            target: target.clone(),
                        },
                        value: stmt.value.clone(),
                        indent: stmt.indent,
                        line: stmt.line,
                        column: stmt.column,
                    };

                    self.variables
                        .insert(name.clone(), Value::Statement(Box::new(pattern_stmt)));

                    println!("📝 Pattern defined: {}", name);
                }

                StatementKind::Bank { name, alias } => {
                    // Handle bank alias: bank devaloop.808 as kit
                    // In WASM mode, banks are pre-registered, so we just create an alias
                    // Look for the bank by name in existing variables

                    let target_alias = alias.clone().unwrap_or_else(|| {
                        // Default alias: use last part of name (e.g., "devaloop.808" -> "808")
                        name.split('.').last().unwrap_or(name).to_string()
                    });

                    // Try to find the bank in variables (it might be registered with a different alias)
                    let mut bank_found = false;

                    // First check if there's already a variable with the target name
                    if let Some(existing_value) = self.variables.get(name) {
                        // Clone the bank data to the new alias
                        self.variables
                            .insert(target_alias.clone(), existing_value.clone());
                        bank_found = true;

                        #[cfg(target_arch = "wasm32")]
                        web_sys::console::log_1(
                            &format!("🏦 Bank alias created: {} -> {}", name, target_alias).into(),
                        );
                        #[cfg(not(target_arch = "wasm32"))]
                        println!("🏦 Bank alias created: {} -> {}", name, target_alias);
                    } else {
                        // Search for bank by full_name in registered banks (WASM mode)
                        #[cfg(target_arch = "wasm32")]
                        {
                            use crate::web::registry::banks::REGISTERED_BANKS;

                            REGISTERED_BANKS.with(|banks| {
                                for bank in banks.borrow().iter() {
                                    if bank.full_name == *name {
                                        // Found the bank! Get its current alias and clone it
                                        if let Some(Value::Map(bank_map)) =
                                            self.variables.get(&bank.alias)
                                        {
                                            self.variables.insert(
                                                target_alias.clone(),
                                                Value::Map(bank_map.clone()),
                                            );
                                            bank_found = true;

                                            web_sys::console::log_1(
                                                &format!(
                                                    "🏦 Bank alias created: {} ({}) -> {}",
                                                    name, bank.alias, target_alias
                                                )
                                                .into(),
                                            );
                                        }
                                    }
                                }
                            });
                        }
                        
                        // Native mode: check if bank exists in sample registry
                        #[cfg(not(target_arch = "wasm32"))]
                        {
                            use std::collections::HashMap;
                            
                            // Create a simple bank map with the bank name
                            let mut bank_map = HashMap::new();
                            bank_map.insert("_name".to_string(), Value::String(name.clone()));
                            bank_map.insert("_alias".to_string(), Value::String(target_alias.clone()));
                            
                            // Store the bank alias
                            self.variables.insert(target_alias.clone(), Value::Map(bank_map));
                            bank_found = true;
                            
                            println!("🏦 Bank alias created: {} -> {}", name, target_alias);
                        }
                    }

                    if !bank_found {
                        #[cfg(target_arch = "wasm32")]
                        web_sys::console::warn_1(&format!("⚠️ Bank not found: {}", name).into());
                        #[cfg(not(target_arch = "wasm32"))]
                        println!("⚠️ Bank not found: {}", name);
                    }
                }

                StatementKind::Bind { source, target } => {
                    // Bind MIDI data to synth
                    self.handle_bind(source, target, &stmt.value)?;
                }

                StatementKind::Trigger {
                    entity,
                    duration: _,
                    effects: _,
                } => {
                    // Play a trigger/sample directly: .kit.kick
                    // Resolve the entity (remove leading dot if present)
                    let resolved_entity = if entity.starts_with('.') {
                        &entity[1..]
                    } else {
                        entity
                    };

                    // Check if it's a nested variable (e.g., "kit.kick")
                    if resolved_entity.contains('.') {
                        let parts: Vec<&str> = resolved_entity.split('.').collect();
                        if parts.len() == 2 {
                            let (var_name, property) = (parts[0], parts[1]);

                            // Try to resolve from variables
                            if let Some(Value::Map(map)) = self.variables.get(var_name) {
                                if let Some(Value::String(sample_uri)) = map.get(property) {
                                    let uri = sample_uri.trim_matches('"').trim_matches('\'');

                                    #[cfg(target_arch = "wasm32")]
                                    web_sys::console::log_1(
                                        &format!(
                                            "🎵 Trigger: {}.{} -> {} at {:.3}s",
                                            var_name, property, uri, self.cursor_time
                                        )
                                        .into(),
                                    );
                                    #[cfg(not(target_arch = "wasm32"))]
                                    println!(
                                        "🎵 Trigger: {}.{} -> {} at {:.3}s",
                                        var_name, property, uri, self.cursor_time
                                    );

                                    // Create and add sample event
                                    self.events.add_sample_event(
                                        uri,
                                        self.cursor_time,
                                        1.0, // Default velocity
                                    );

                                    // Advance cursor_time by 1 beat for sequential playback (respects BPM)
                                    let beat_duration = self.beat_duration();
                                    self.cursor_time += beat_duration;

                                    #[cfg(target_arch = "wasm32")]
                                    web_sys::console::log_1(&format!("   Next trigger at {:.3}s (advanced by 1 beat = {:.3}s)", 
                                        self.cursor_time, beat_duration).into());
                                }
                            }
                        }
                    } else {
                        // Check if it's a direct sample/trigger variable
                        if let Some(Value::String(sample_uri)) = self.variables.get(resolved_entity)
                        {
                            let uri = sample_uri.trim_matches('"').trim_matches('\'');

                            #[cfg(target_arch = "wasm32")]
                            web_sys::console::log_1(
                                &format!(
                                    "🎵 Trigger: {} -> {} at {:.3}s",
                                    resolved_entity, uri, self.cursor_time
                                )
                                .into(),
                            );
                            #[cfg(not(target_arch = "wasm32"))]
                            println!(
                                "🎵 Trigger: {} -> {} at {:.3}s",
                                resolved_entity, uri, self.cursor_time
                            );

                            // Create and add sample event
                            self.events.add_sample_event(
                                uri,
                                self.cursor_time,
                                1.0, // Default velocity
                            );

                            // Advance cursor_time by 1 beat for sequential playback (respects BPM)
                            let beat_duration = self.beat_duration();
                            self.cursor_time += beat_duration;

                            #[cfg(target_arch = "wasm32")]
                            web_sys::console::log_1(
                                &format!(
                                    "   Next trigger at {:.3}s (advanced by 1 beat = {:.3}s)",
                                    self.cursor_time, beat_duration
                                )
                                .into(),
                            );
                        }
                    }
                }

                _ => {
                    // Other statements not yet implemented
                }
            }
        }

        // Execute spawns in parallel using rayon
        if !spawns.is_empty() {
            println!("🚀 Executing {} spawn(s) in parallel", spawns.len());

            // Capture current state for parallel execution
            let current_time = self.cursor_time;
            let current_bpm = self.bpm;
            let groups_snapshot = self.groups.clone();
            let variables_snapshot = self.variables.clone();
            let special_vars_snapshot = self.special_vars.clone();

            // Execute all spawns in parallel and collect results
            let spawn_results: Vec<Result<AudioEventList>> = spawns
                .par_iter()
                .map(|stmt| {
                    if let StatementKind::Spawn { name, args: _ } = &stmt.kind {
                        // Resolve variable name (remove leading dot if present)
                        let resolved_name = if name.starts_with('.') {
                            &name[1..]
                        } else {
                            name
                        };
                        // Check if it's a nested variable (e.g., "kit.kick")
                        if resolved_name.contains('.') {
                            let parts: Vec<&str> = resolved_name.split('.').collect();
                            if parts.len() == 2 {
                                let (var_name, property) = (parts[0], parts[1]);
                                // Try to resolve from variables
                                if let Some(Value::Map(map)) = variables_snapshot.get(var_name) {
                                    if let Some(Value::String(sample_uri)) = map.get(property) {
                                        println!("🎵 Spawn nested sample: {}.{} -> {}", var_name, property, sample_uri);
                                        let mut event_list = AudioEventList::new();
                                        event_list.add_sample_event(
                                            sample_uri.trim_matches('"').trim_matches('\''),
                                            current_time,
                                            1.0, // Default velocity
                                        );
                                        return Ok(event_list);
                                    }
                                }
                            }
                        }
                        // Check if it's a direct sample/trigger variable
                        if let Some(sample_value) = variables_snapshot.get(resolved_name) {
                            if let Value::String(sample_uri) = sample_value {
                                println!("🎵 Spawn sample: {} -> {}", resolved_name, sample_uri);
                                // Create a sample playback event
                                let mut event_list = AudioEventList::new();
                                event_list.add_sample_event(
                                    sample_uri.trim_matches('"').trim_matches('\''),
                                    current_time,
                                    1.0, // Default velocity
                                );
                                return Ok(event_list);
                            }
                        }
                        // Create a new interpreter instance for each spawn (group)
                        let mut local_interpreter = AudioInterpreter {
                            sample_rate: self.sample_rate,
                            bpm: current_bpm,
                            function_registry: FunctionRegistry::new(), // Create new instance instead of clone
                            events: AudioEventList::new(),
                            variables: variables_snapshot.clone(),
                            groups: groups_snapshot.clone(),
                            cursor_time: current_time,
                            special_vars: special_vars_snapshot.clone(),
                            event_registry: EventRegistry::new(),
                            current_statement_location: None, // Spawn doesn't need statement tracking
                        };
                        // Execute the spawned group
                        if let Some(body) = groups_snapshot.get(resolved_name) {
                            println!("🎬 Spawn group: {} (parallel)", resolved_name);
                            local_interpreter.collect_events(body)?;
                            Ok(local_interpreter.events)
                        } else {
                            println!("⚠️  Warning: Spawn target '{}' not found (neither sample nor group)", resolved_name);
                            Ok(AudioEventList::new())
                        }
                    } else {
                        Ok(AudioEventList::new())
                    }
                })
                .collect();

            // Merge all spawn results into main event list
            for result in spawn_results {
                match result {
                    Ok(spawn_events) => {
                        self.events.merge(spawn_events);
                    }
                    Err(e) => {
                        println!("⚠️  Error in spawn execution: {}", e);
                        // Continue with other spawns even if one fails
                    }
                }
            }

            println!("✅ Parallel spawn execution completed");
        }

        Ok(())
    }

    fn handle_let(&mut self, name: &str, value: &Value) -> Result<()> {
        // Check if this is a synth definition (has waveform parameter)
        if let Value::Map(map) = value {
            if map.contains_key("waveform") {
                // Extract synth parameters
                let waveform = extract_string(map, "waveform", "sine");
                let attack = extract_number(map, "attack", 0.01);
                let decay = extract_number(map, "decay", 0.1);
                let sustain = extract_number(map, "sustain", 0.7);
                let release = extract_number(map, "release", 0.2);

                // Extract synth type (pluck, arp, pad, etc.)
                let synth_type = if let Some(Value::String(t)) = map.get("type") {
                    // Remove quotes if present (parser includes them)
                    let clean = t.trim_matches('"').trim_matches('\'');
                    if clean.is_empty() || clean == "synth" {
                        None
                    } else {
                        Some(clean.to_string())
                    }
                } else {
                    None
                };

                // Extract filters array
                let filters = if let Some(Value::Array(filters_arr)) = map.get("filters") {
                    extract_filters(filters_arr)
                } else {
                    Vec::new()
                };

                // Extract options (gate, click_amount, rate, etc.)
                let mut options = std::collections::HashMap::new();
                for (key, val) in map.iter() {
                    if ![
                        "waveform", "attack", "decay", "sustain", "release", "type", "filters",
                    ]
                    .contains(&key.as_str())
                    {
                        if let Value::Number(n) = val {
                            options.insert(key.clone(), *n);
                        }
                    }
                }

                let type_info = synth_type
                    .as_ref()
                    .map(|t| format!(" [{}]", t))
                    .unwrap_or_default();
                let opts_info = if !options.is_empty() {
                    let opts: Vec<String> = options
                        .iter()
                        .map(|(k, v)| format!("{}={:.2}", k, v))
                        .collect();
                    format!(" (options: {})", opts.join(", "))
                } else {
                    String::new()
                };

                let synth_def = SynthDefinition {
                    waveform,
                    attack,
                    decay,
                    sustain,
                    release,
                    synth_type,
                    filters,
                    options,
                };

                self.events.add_synth(name.to_string(), synth_def);

                println!("🎹 Synth registered: {}{}{}", name, type_info, opts_info);
            }
        }

        // Store variable
        self.variables.insert(name.to_string(), value.clone());
        Ok(())
    }

    fn extract_audio_event(
        &mut self,
        target: &str,
        context: &crate::engine::functions::FunctionContext,
    ) -> Result<()> {
        // Check if there's a note event
        if let Some(Value::String(note_name)) = context.get("note") {
            let midi = parse_note_to_midi(note_name)?;
            let duration = if let Some(Value::Number(d)) = context.get("duration") {
                d / 1000.0 // Convert ms to seconds
            } else {
                0.5
            };
            let velocity = if let Some(Value::Number(v)) = context.get("velocity") {
                v / 100.0 // Normalize 0-100 to 0.0-1.0
            } else {
                0.8
            };

            // Extract audio options
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

            let attack = if let Some(Value::Number(a)) = context.get("attack") {
                Some(*a)
            } else {
                None
            };

            let release = if let Some(Value::Number(r)) = context.get("release") {
                Some(*r)
            } else {
                None
            };

            // Extract effects parameters
            let delay_time = if let Some(Value::Number(t)) = context.get("delay_time") {
                Some(*t)
            } else {
                None
            };

            let delay_feedback = if let Some(Value::Number(f)) = context.get("delay_feedback") {
                Some(*f)
            } else {
                None
            };

            let delay_mix = if let Some(Value::Number(m)) = context.get("delay_mix") {
                Some(*m)
            } else {
                None
            };

            let reverb_amount = if let Some(Value::Number(a)) = context.get("reverb_amount") {
                Some(*a)
            } else {
                None
            };

            let drive_amount = if let Some(Value::Number(a)) = context.get("drive_amount") {
                Some(*a)
            } else {
                None
            };

            let drive_color = if let Some(Value::Number(c)) = context.get("drive_color") {
                Some(*c)
            } else {
                None
            };

            self.events.add_note_event(
                target,
                midi,
                context.start_time,
                duration,
                velocity,
                pan,
                detune,
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

            // Generate playhead events (note on/off)
            #[cfg(target_arch = "wasm32")]
            {
                use crate::web::registry::playhead::{PlayheadEvent, push_event};

                // Note on event
                push_event(PlayheadEvent {
                    event_type: "note_on".to_string(),
                    midi: vec![midi],
                    time: context.start_time,
                    velocity,
                    synth_id: target.to_string(),
                });

                // Note off event
                push_event(PlayheadEvent {
                    event_type: "note_off".to_string(),
                    midi: vec![midi],
                    time: context.start_time + duration,
                    velocity,
                    synth_id: target.to_string(),
                });
            }
        }

        // Check if there's a chord event
        if let Some(Value::Array(notes)) = context.get("notes") {
            let mut midis = Vec::new();
            for note_val in notes {
                if let Value::String(note_name) = note_val {
                    midis.push(parse_note_to_midi(note_name)?);
                }
            }

            let duration = if let Some(Value::Number(d)) = context.get("duration") {
                d / 1000.0
            } else {
                0.5
            };
            let velocity = if let Some(Value::Number(v)) = context.get("velocity") {
                v / 100.0
            } else {
                0.8
            };

            // Extract audio options
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

            let attack = if let Some(Value::Number(a)) = context.get("attack") {
                Some(*a)
            } else {
                None
            };

            let release = if let Some(Value::Number(r)) = context.get("release") {
                Some(*r)
            } else {
                None
            };

            // Extract effects parameters
            let delay_time = if let Some(Value::Number(t)) = context.get("delay_time") {
                Some(*t)
            } else {
                None
            };

            let delay_feedback = if let Some(Value::Number(f)) = context.get("delay_feedback") {
                Some(*f)
            } else {
                None
            };

            let delay_mix = if let Some(Value::Number(m)) = context.get("delay_mix") {
                Some(*m)
            } else {
                None
            };

            let reverb_amount = if let Some(Value::Number(a)) = context.get("reverb_amount") {
                Some(*a)
            } else {
                None
            };

            let drive_amount = if let Some(Value::Number(a)) = context.get("drive_amount") {
                Some(*a)
            } else {
                None
            };

            let drive_color = if let Some(Value::Number(c)) = context.get("drive_color") {
                Some(*c)
            } else {
                None
            };

            self.events.add_chord_event(
                target,
                midis.clone(),
                context.start_time,
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

            // Generate playhead events (chord on/off)
            #[cfg(target_arch = "wasm32")]
            {
                use crate::web::registry::playhead::{PlayheadEvent, push_event};

                // Chord on event
                push_event(PlayheadEvent {
                    event_type: "chord_on".to_string(),
                    midi: midis.clone(),
                    time: context.start_time,
                    velocity,
                    synth_id: target.to_string(),
                });

                // Chord off event
                push_event(PlayheadEvent {
                    event_type: "chord_off".to_string(),
                    midi: midis,
                    time: context.start_time + duration,
                    velocity,
                    synth_id: target.to_string(),
                });
            }
        }

        Ok(())
    }

    fn render_audio(&self) -> Result<Vec<f32>> {
        let total_duration = self.events.total_duration();
        if total_duration <= 0.0 {
            return Ok(Vec::new());
        }

        let total_samples = (total_duration * self.sample_rate as f32).ceil() as usize;
        let mut buffer = vec![0.0f32; total_samples * 2]; // stereo

        // Count events by type
        let mut _note_count = 0;
        let mut _chord_count = 0;
        let mut _sample_count = 0;
        for event in &self.events.events {
            match event {
                crate::engine::audio::events::AudioEvent::Note { .. } => _note_count += 1,
                crate::engine::audio::events::AudioEvent::Chord { .. } => _chord_count += 1,
                crate::engine::audio::events::AudioEvent::Sample { .. } => _sample_count += 1,
            }
        }

        // Render each event
        for event in &self.events.events {
            match event {
                crate::engine::audio::events::AudioEvent::Note {
                    midi,
                    start_time,
                    duration,
                    velocity,
                    synth_id: _,
                    synth_def,
                    pan,
                    detune,
                    gain,
                    attack,
                    release,
                    delay_time,
                    delay_feedback,
                    delay_mix,
                    reverb_amount,
                    drive_amount,
                    drive_color,
                } => {
                    // Use captured synth definition from event creation time
                    let mut params = SynthParams {
                        waveform: synth_def.waveform.clone(),
                        attack: synth_def.attack,
                        decay: synth_def.decay,
                        sustain: synth_def.sustain,
                        release: synth_def.release,
                        synth_type: synth_def.synth_type.clone(),
                        filters: synth_def.filters.clone(),
                        options: synth_def.options.clone(),
                    };

                    // Override attack/release if specified
                    if let Some(a) = attack {
                        params.attack = a / 1000.0; // Convert ms to seconds
                    }
                    if let Some(r) = release {
                        params.release = r / 1000.0; // Convert ms to seconds
                    }

                    let mut samples = generate_note_with_options(
                        *midi,
                        duration * 1000.0,
                        velocity * gain, // Apply gain to velocity
                        &params,
                        self.sample_rate,
                        *pan,
                        *detune,
                    )?;

                    // Apply effects in order: Drive -> Reverb -> Delay

                    // Apply drive (saturation)
                    if let Some(amount) = drive_amount {
                        let color = drive_color.unwrap_or(0.5);
                        let mix = 0.7; // Drive mix
                        let mut processor = DriveProcessor::new(*amount, color, mix);
                        processor.process(&mut samples, self.sample_rate);
                    }

                    // Apply reverb
                    if let Some(amount) = reverb_amount {
                        let room_size = *amount; // Use amount as room size
                        let damping = 0.5; // Default damping
                        let mix = *amount * 0.5; // Scale mix with amount
                        let mut processor = ReverbProcessor::new(room_size, damping, mix);
                        processor.process(&mut samples, self.sample_rate);
                    }

                    // Apply delay
                    if let Some(time) = delay_time {
                        let feedback = delay_feedback.unwrap_or(0.3);
                        let mix = delay_mix.unwrap_or(0.5);
                        let mut processor = DelayProcessor::new(*time, feedback, mix);
                        processor.process(&mut samples, self.sample_rate);
                    }

                    // Mix into buffer
                    let start_sample = (*start_time * self.sample_rate as f32) as usize * 2;
                    for (i, &sample) in samples.iter().enumerate() {
                        let buf_idx = start_sample + i;
                        if buf_idx < buffer.len() {
                            buffer[buf_idx] += sample;
                        }
                    }
                }

                crate::engine::audio::events::AudioEvent::Chord {
                    midis,
                    start_time,
                    duration,
                    velocity,
                    synth_id: _,
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
                } => {
                    // Use captured synth definition from event creation time
                    let mut params = SynthParams {
                        waveform: synth_def.waveform.clone(),
                        attack: synth_def.attack,
                        decay: synth_def.decay,
                        sustain: synth_def.sustain,
                        release: synth_def.release,
                        synth_type: synth_def.synth_type.clone(),
                        filters: synth_def.filters.clone(),
                        options: synth_def.options.clone(),
                    };

                    // Override attack/release if specified
                    if let Some(a) = attack {
                        params.attack = a / 1000.0; // Convert ms to seconds
                    }
                    if let Some(r) = release {
                        params.release = r / 1000.0; // Convert ms to seconds
                    }

                    let mut samples = generate_chord_with_options(
                        midis,
                        duration * 1000.0,
                        velocity * gain, // Apply gain to velocity
                        &params,
                        self.sample_rate,
                        *pan,
                        *detune,
                        *spread,
                    )?;

                    // Apply effects in order: Drive -> Reverb -> Delay

                    // Apply drive (saturation)
                    if let Some(amount) = drive_amount {
                        let color = drive_color.unwrap_or(0.5);
                        let mix = 0.7; // Drive mix
                        let mut processor = DriveProcessor::new(*amount, color, mix);
                        processor.process(&mut samples, self.sample_rate);
                    }

                    // Apply reverb
                    if let Some(amount) = reverb_amount {
                        let room_size = *amount; // Use amount as room size
                        let damping = 0.5; // Default damping
                        let mix = *amount * 0.5; // Scale mix with amount
                        let mut processor = ReverbProcessor::new(room_size, damping, mix);
                        processor.process(&mut samples, self.sample_rate);
                    }

                    // Apply delay
                    if let Some(time) = delay_time {
                        let feedback = delay_feedback.unwrap_or(0.3);
                        let mix = delay_mix.unwrap_or(0.5);
                        let mut processor = DelayProcessor::new(*time, feedback, mix);
                        processor.process(&mut samples, self.sample_rate);
                    }

                    // Mix into buffer
                    let start_sample = (*start_time * self.sample_rate as f32) as usize * 2;
                    for (i, &sample) in samples.iter().enumerate() {
                        let buf_idx = start_sample + i;
                        if buf_idx < buffer.len() {
                            buffer[buf_idx] += sample;
                        }
                    }
                }

                crate::engine::audio::events::AudioEvent::Sample {
                    uri,
                    start_time,
                    velocity,
                } => {
                    // Get sample PCM data from registry (WASM feature)
                    #[cfg(target_arch = "wasm32")]
                    {
                        use crate::web::registry::samples::get_sample;

                        if let Some(pcm_data) = get_sample(uri) {
                            // Convert i16 PCM to f32 and apply velocity
                            let start_sample_idx = (*start_time * self.sample_rate as f32) as usize;

                            web_sys::console::log_1(
                                &format!(
                                    "🔊 Rendering sample: {} at {:.3}s, {} frames, velocity {:.2}",
                                    uri,
                                    start_time,
                                    pcm_data.len(),
                                    velocity
                                )
                                .into(),
                            );
                            web_sys::console::log_1(
                                &format!(
                                    "   Start buffer index: {} (stereo pos: {})",
                                    start_sample_idx,
                                    start_sample_idx * 2
                                )
                                .into(),
                            );

                            // Assume mono samples - duplicate to stereo
                            for (i, &pcm_value) in pcm_data.iter().enumerate() {
                                // Convert i16 to f32 (-1.0 to 1.0) and apply velocity
                                let sample = (pcm_value as f32 / 32768.0) * velocity;

                                // Calculate buffer positions for stereo (multiply by 2 because stereo)
                                let stereo_pos = (start_sample_idx + i) * 2;
                                let buf_idx_l = stereo_pos;
                                let buf_idx_r = stereo_pos + 1;

                                // Mix into buffer (duplicate mono to both channels)
                                if buf_idx_l < buffer.len() {
                                    buffer[buf_idx_l] += sample;
                                }
                                if buf_idx_r < buffer.len() {
                                    buffer[buf_idx_r] += sample;
                                }
                            }
                        } else {
                            println!("⚠️  Sample not found in registry: {}", uri);
                        }
                    }

                    // For native builds, try to load sample from registry
                    #[cfg(not(target_arch = "wasm32"))]
                    {
                        use crate::engine::audio::samples;
                        
                        if let Some(sample_data) = samples::get_sample(uri) {
                            // Sample found in registry, render it
                            let start_sample_idx = (*start_time * self.sample_rate as f32) as usize;
                            let velocity_scale = velocity / 100.0;
                            
                            // Resample if needed
                            let resample_ratio = self.sample_rate as f32 / sample_data.sample_rate as f32;
                            
                            for (i, &sample) in sample_data.samples.iter().enumerate() {
                                // Calculate output position considering resampling
                                let output_idx = start_sample_idx + (i as f32 * resample_ratio) as usize;
                                let stereo_pos = output_idx * 2;
                                let buf_idx_l = stereo_pos;
                                let buf_idx_r = stereo_pos + 1;
                                
                                let scaled_sample = sample * velocity_scale;
                                
                                // Mix into stereo buffer
                                if buf_idx_l < buffer.len() {
                                    buffer[buf_idx_l] += scaled_sample;
                                }
                                if buf_idx_r < buffer.len() {
                                    buffer[buf_idx_r] += scaled_sample;
                                }
                            }
                        } else {
                            // Sample not found in registry - log error
                            eprintln!("❌ Error: Bank sample not found: {}", uri);
                            eprintln!("   Make sure the bank is loaded and the sample exists.");
                            // Note: No audio will be rendered for this sample
                        }
                    }
                }
            }
        }

        // Normalize to prevent clipping
        let max_amplitude = buffer.iter().map(|&s| s.abs()).fold(0.0f32, f32::max);
        if max_amplitude > 1.0 {
            for sample in buffer.iter_mut() {
                *sample /= max_amplitude;
            }
        }

        Ok(buffer)
    }

    pub fn set_bpm(&mut self, bpm: f32) {
        self.bpm = bpm.max(1.0).min(999.0);
    }

    pub fn samples_per_beat(&self) -> usize {
        ((60.0 / self.bpm) * self.sample_rate as f32) as usize
    }

    /// Get duration of one beat in seconds
    pub fn beat_duration(&self) -> f32 {
        60.0 / self.bpm
    }

    /// Execute print statement with variable interpolation
    /// Supports {variable_name} syntax
    fn execute_print(&self, value: &Value) -> Result<()> {
        let message = match value {
            Value::String(s) => {
                // Check if string contains variable interpolation
                if s.contains('{') && s.contains('}') {
                    self.interpolate_string(s)
                } else {
                    s.clone()
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

    /// Interpolate variables in a string
    /// Replaces {variable_name} with the variable's value
    fn interpolate_string(&self, template: &str) -> String {
        let mut result = template.to_string();

        // Find all {variable} patterns
        let re = regex::Regex::new(r"\{([a-zA-Z_][a-zA-Z0-9_]*)\}").unwrap();

        for cap in re.captures_iter(template) {
            let full_match = &cap[0]; // {variable_name}
            let var_name = &cap[1]; // variable_name

            if let Some(value) = self.variables.get(var_name) {
                let replacement = self.value_to_string(value);
                result = result.replace(full_match, &replacement);
            } else {
                // Variable not found, leave placeholder or show error
                result = result.replace(full_match, &format!("<undefined:{}>", var_name));
            }
        }

        result
    }

    /// Convert a Value to a displayable string
    fn value_to_string(&self, value: &Value) -> String {
        match value {
            Value::String(s) => {
                // Remove surrounding quotes if present
                s.trim_matches('"').trim_matches('\'').to_string()
            }
            Value::Number(n) => {
                // Format nicely: remove trailing zeros
                if n.fract() == 0.0 {
                    format!("{:.0}", n)
                } else {
                    format!("{}", n)
                }
            }
            Value::Boolean(b) => b.to_string(),
            Value::Array(arr) => {
                let items: Vec<String> = arr.iter().map(|v| self.value_to_string(v)).collect();
                format!("[{}]", items.join(", "))
            }
            Value::Identifier(id) => id.clone(),
            _ => format!("{:?}", value),
        }
    }

    /// Execute if statement with condition evaluation
    fn execute_if(
        &mut self,
        condition: &Value,
        body: &[Statement],
        else_body: &Option<Vec<Statement>>,
    ) -> Result<()> {
        let condition_result = self.evaluate_condition(condition)?;

        if condition_result {
            // Condition is true, execute if body
            self.collect_events(body)?;
        } else if let Some(else_stmts) = else_body {
            // Condition is false, execute else body
            self.collect_events(else_stmts)?;
        }

        Ok(())
    }

    /// Evaluate a condition to a boolean
    /// Supports: ==, !=, <, >, <=, >=
    fn evaluate_condition(&self, condition: &Value) -> Result<bool> {
        // Condition is stored as a Map with operator and operands
        if let Value::Map(map) = condition {
            let operator = map
                .get("operator")
                .and_then(|v| {
                    if let Value::String(s) = v {
                        Some(s.as_str())
                    } else {
                        None
                    }
                })
                .unwrap_or("==");

            let left = map
                .get("left")
                .ok_or_else(|| anyhow::anyhow!("Missing left operand"))?;
            let right = map
                .get("right")
                .ok_or_else(|| anyhow::anyhow!("Missing right operand"))?;

            // Resolve values (replace identifiers with their values)
            let left_val = self.resolve_value(left);
            let right_val = self.resolve_value(right);

            // Compare based on operator
            match operator {
                "==" => Ok(self.values_equal(&left_val, &right_val)),
                "!=" => Ok(!self.values_equal(&left_val, &right_val)),
                "<" => self.compare_values(&left_val, &right_val, std::cmp::Ordering::Less),
                ">" => self.compare_values(&left_val, &right_val, std::cmp::Ordering::Greater),
                "<=" => {
                    let less =
                        self.compare_values(&left_val, &right_val, std::cmp::Ordering::Less)?;
                    let equal = self.values_equal(&left_val, &right_val);
                    Ok(less || equal)
                }
                ">=" => {
                    let greater =
                        self.compare_values(&left_val, &right_val, std::cmp::Ordering::Greater)?;
                    let equal = self.values_equal(&left_val, &right_val);
                    Ok(greater || equal)
                }
                _ => Err(anyhow::anyhow!("Unknown operator: {}", operator)),
            }
        } else {
            // Direct boolean value
            match condition {
                Value::Boolean(b) => Ok(*b),
                Value::Number(n) => Ok(*n != 0.0),
                Value::String(s) => Ok(!s.is_empty()),
                _ => Ok(false),
            }
        }
    }

    /// Resolve a value (replace identifiers with their values from variables)
    fn resolve_value(&self, value: &Value) -> Value {
        match value {
            Value::Identifier(name) => {
                // Check if it's a special variable
                if is_special_var(name) {
                    if let Some(special_val) = resolve_special_var(name, &self.special_vars) {
                        return special_val;
                    }
                }

                // Otherwise, look in variables
                self.variables.get(name).cloned().unwrap_or(Value::Null)
            }
            _ => value.clone(),
        }
    }

    /// Execute event handlers matching the event name
    fn execute_event_handlers(&mut self, event_name: &str) -> Result<()> {
        let handlers = self.event_registry.get_handlers_matching(event_name);

        for (index, handler) in handlers.iter().enumerate() {
            // Check if this is a "once" handler that has already been executed
            if handler.once && !self.event_registry.should_execute_once(event_name, index) {
                continue;
            }

            // Execute the handler body
            let body_clone = handler.body.clone();
            self.collect_events(&body_clone)?;
        }

        Ok(())
    }

    /// Check if two values are equal
    fn values_equal(&self, left: &Value, right: &Value) -> bool {
        match (left, right) {
            (Value::Number(a), Value::Number(b)) => (a - b).abs() < 0.0001,
            (Value::String(a), Value::String(b)) => a == b,
            (Value::Boolean(a), Value::Boolean(b)) => a == b,
            (Value::Null, Value::Null) => true,
            _ => false,
        }
    }

    /// Compare two values
    fn compare_values(
        &self,
        left: &Value,
        right: &Value,
        ordering: std::cmp::Ordering,
    ) -> Result<bool> {
        match (left, right) {
            (Value::Number(a), Value::Number(b)) => {
                Ok(a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal) == ordering)
            }
            (Value::String(a), Value::String(b)) => Ok(a.cmp(b) == ordering),
            _ => Err(anyhow::anyhow!("Cannot compare {:?} and {:?}", left, right)),
        }
    }

    /// Handle property assignment: target.property = value
    fn handle_assign(&mut self, target: &str, property: &str, value: &Value) -> Result<()> {
        // Get the variable (which should be a Map for synth definitions)
        if let Some(var) = self.variables.get_mut(target) {
            if let Value::Map(map) = var {
                // Update the property in the map
                map.insert(property.to_string(), value.clone());

                // If this is a synth and the property affects the definition, update it
                if self.events.synths.contains_key(target) {
                    // Clone the map to avoid borrow issues
                    let map_clone = map.clone();
                    // Update synth definition
                    let updated_def = self.extract_synth_def_from_map(&map_clone)?;
                    self.events.synths.insert(target.to_string(), updated_def);
                    println!("🔧 Updated {}.{} = {:?}", target, property, value);
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

        Ok(())
    }

    /// Extract synth definition from a map
    fn extract_synth_def_from_map(&self, map: &HashMap<String, Value>) -> Result<SynthDefinition> {
        use crate::engine::audio::events::extract_filters;

        let waveform = extract_string(map, "waveform", "sine");
        let attack = extract_number(map, "attack", 0.01);
        let decay = extract_number(map, "decay", 0.1);
        let sustain = extract_number(map, "sustain", 0.7);
        let release = extract_number(map, "release", 0.2);

        let synth_type = if let Some(Value::String(t)) = map.get("type") {
            let clean = t.trim_matches('"').trim_matches('\'');
            if clean.is_empty() || clean == "synth" {
                None
            } else {
                Some(clean.to_string())
            }
        } else {
            None
        };

        let filters = if let Some(Value::Array(filters_arr)) = map.get("filters") {
            extract_filters(filters_arr)
        } else {
            Vec::new()
        };

        let mut options = HashMap::new();
        for (key, val) in map.iter() {
            if ![
                "waveform", "attack", "decay", "sustain", "release", "type", "filters",
            ]
            .contains(&key.as_str())
            {
                if let Value::Number(n) = val {
                    options.insert(key.clone(), *n);
                }
            }
        }

        Ok(SynthDefinition {
            waveform,
            attack,
            decay,
            sustain,
            release,
            synth_type,
            filters,
            options,
        })
    }

    /// Handle MIDI file loading: @load "path.mid" as alias
    fn handle_load(&mut self, source: &str, alias: &str) -> Result<()> {
        use crate::engine::audio::midi::load_midi_file;
        use std::path::Path;

        // Load the MIDI file
        let path = Path::new(source);
        let midi_data = load_midi_file(path)?;

        // Store in variables
        self.variables.insert(alias.to_string(), midi_data);

        println!("🎵 Loaded MIDI file: {} as {}", source, alias);

        Ok(())
    }

    /// Handle MIDI binding: bind source -> target { options }
    fn handle_bind(&mut self, source: &str, target: &str, options: &Value) -> Result<()> {
        // Get MIDI data from variables
        let midi_data = self
            .variables
            .get(source)
            .ok_or_else(|| anyhow::anyhow!("MIDI source '{}' not found", source))?
            .clone();

        // Extract notes from MIDI data
        if let Value::Map(midi_map) = &midi_data {
            let notes = midi_map
                .get("notes")
                .ok_or_else(|| anyhow::anyhow!("MIDI data has no notes"))?;

            if let Value::Array(notes_array) = notes {
                // Get synth definition (for validation)
                let _synth_def = self
                    .events
                    .synths
                    .get(target)
                    .ok_or_else(|| anyhow::anyhow!("Synth '{}' not found", target))?
                    .clone();

                // Extract options (velocity, bpm, etc.)
                let default_velocity = 100;
                let mut velocity = default_velocity;

                if let Value::Map(opts) = options {
                    if let Some(Value::Number(v)) = opts.get("velocity") {
                        velocity = *v as u8;
                    }
                }

                // Generate events for each note
                for note_val in notes_array {
                    if let Value::Map(note_map) = note_val {
                        let time = extract_number(note_map, "time", 0.0);
                        let note = extract_number(note_map, "note", 60.0) as u8;
                        let note_velocity =
                            extract_number(note_map, "velocity", velocity as f32) as u8;

                        // Create audio event
                        use crate::engine::audio::events::AudioEvent;
                        // Capture synth definition snapshot
                        let synth_def = self.events.get_synth(target).cloned().unwrap_or_default();
                        
                        let event = AudioEvent::Note {
                            midi: note,
                            start_time: time / 1000.0, // Convert from ticks to seconds (simplified)
                            duration: 0.5,             // Default duration, could be in options
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

                        self.events.events.push(event);
                    }
                }

                println!(
                    "🔗 Bound {} notes from {} to {}",
                    notes_array.len(),
                    source,
                    target
                );
            }
        }

        Ok(())
    }

    /// Extract pattern string and options from pattern value
    fn extract_pattern_data(&self, value: &Value) -> (Option<String>, Option<HashMap<String, f32>>) {
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

                // Extract numeric options (swing, humanize, velocity, tempo)
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

    /// Execute a pattern with given target and pattern string
    fn execute_pattern(
        &mut self,
        target: &str,
        pattern: &str,
        options: Option<HashMap<String, f32>>,
    ) -> Result<()> {
        use crate::engine::audio::events::AudioEvent;

        // Extract options or use defaults
        let swing = options.as_ref().and_then(|o| o.get("swing").copied()).unwrap_or(0.0);
        let humanize = options.as_ref().and_then(|o| o.get("humanize").copied()).unwrap_or(0.0);
        let velocity_mult = options.as_ref().and_then(|o| o.get("velocity").copied()).unwrap_or(1.0);
        let tempo_override = options.as_ref().and_then(|o| o.get("tempo").copied());

        // Use tempo override or default BPM
        let effective_bpm = tempo_override.unwrap_or(self.bpm);

        // Resolve target URI (e.g., myBank.kick -> devalang://bank/devaloop.808/kick)
        let resolved_uri = self.resolve_sample_uri(target);

        // Remove whitespace from pattern
        let pattern_chars: Vec<char> = pattern.chars().filter(|c| !c.is_whitespace()).collect();
        let step_count = pattern_chars.len() as f32;

        if step_count == 0.0 {
            return Ok(());
        }

        // Calculate step duration based on BPM
        // Assume 4 beats per bar (4/4 time signature)
        let bar_duration = (60.0 / effective_bpm) * 4.0;
        let step_duration = bar_duration / step_count;

        // Generate trigger events for each step
        for (i, &ch) in pattern_chars.iter().enumerate() {
            if ch == 'x' || ch == 'X' {
                // Calculate time with swing
                let mut time = self.cursor_time + (i as f32 * step_duration);

                // Apply swing to every other step
                if swing > 0.0 && i % 2 == 1 {
                    time += step_duration * swing;
                }

                // Apply humanization (random timing offset)
                if humanize > 0.0 {
                    use rand::Rng;
                    let mut rng = rand::thread_rng();
                    let offset = rng.gen_range(-humanize..humanize);
                    time += offset;
                }

                // Create sample event for trigger
                let event = AudioEvent::Sample {
                    uri: resolved_uri.clone(),
                    start_time: time,
                    velocity: 100.0 * velocity_mult,
                };

                self.events.events.push(event);
            }
        }

        // Update cursor time to end of pattern
        self.cursor_time += bar_duration;

        Ok(())
    }

    /// Resolve sample URI from bank.trigger notation (e.g., myBank.kick -> devalang://bank/devaloop.808/kick)
    fn resolve_sample_uri(&self, target: &str) -> String {
        // Split target by '.' to get bank_alias and trigger_name
        if let Some(dot_pos) = target.find('.') {
            let bank_alias = &target[..dot_pos];
            let trigger_name = &target[dot_pos + 1..];

            // Look up bank_alias in variables to get actual bank name
            if let Some(Value::Map(bank_map)) = self.variables.get(bank_alias) {
                if let Some(Value::String(bank_name)) = bank_map.get("_name") {
                    // Construct the devalang URI
                    return format!("devalang://bank/{}/{}", bank_name, trigger_name);
                }
            }
        }

        // Fallback: return original target if resolution fails
        target.to_string()
    }
}

