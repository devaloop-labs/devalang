use crate::engine::audio::events::AudioEventList;
use crate::engine::audio::events::SynthDefinition;
#[cfg(feature = "cli")]
use crate::engine::audio::midi_native::MidiManager;
use crate::engine::events::EventRegistry;
use crate::engine::functions::FunctionRegistry;
use crate::engine::special_vars::{SpecialVarContext, is_special_var, resolve_special_var};
#[cfg(feature = "cli")]
use crate::language::addons::registry::BankRegistry;
use crate::language::syntax::ast::{Statement, Value};

#[cfg(not(feature = "cli"))]
#[allow(dead_code)]
#[derive(Clone)]
pub struct BankRegistry;

#[cfg(not(feature = "cli"))]
impl BankRegistry {
    pub fn new() -> Self {
        BankRegistry
    }
    pub fn list_banks(&self) -> Vec<(String, StubBank)> {
        Vec::new()
    }
    pub fn resolve_trigger(&self, _var: &str, _prop: &str) -> Option<std::path::PathBuf> {
        None
    }
    pub fn register_bank(
        &self,
        _alias: String,
        _name: &str,
        _cwd: &std::path::Path,
        _cwd2: &std::path::Path,
    ) -> Result<(), anyhow::Error> {
        Ok(())
    }
}

#[cfg(not(feature = "cli"))]
#[allow(dead_code)]
#[derive(Clone)]
pub struct StubBank;

#[cfg(not(feature = "cli"))]
impl StubBank {
    pub fn list_triggers(&self) -> Vec<String> {
        Vec::new()
    }
}

/// Context for note-mode automations: contains templates and their temporal bounds
#[derive(Clone, Debug)]
pub struct NoteAutomationContext {
    pub templates: Vec<crate::engine::audio::automation::AutomationParamTemplate>,
    pub start_time: f32,
    pub end_time: f32,
}

impl NoteAutomationContext {
    /// Calculate the total duration of this automation block
    pub fn duration(&self) -> f32 {
        (self.end_time - self.start_time).max(0.001) // Avoid division by zero
    }

    /// Calculate global progress (0.0 to 1.0) for a given time point within this block
    pub fn progress_at_time(&self, time: f32) -> f32 {
        ((time - self.start_time) / self.duration()).clamp(0.0, 1.0)
    }
}

/// Audio interpreter driver - main execution loop
use anyhow::Result;
use std::collections::HashMap;

pub mod collector;
pub mod extractor;
pub mod handler;
pub mod renderer;

pub struct AudioInterpreter {
    pub sample_rate: u32,
    pub bpm: f32,
    pub function_registry: FunctionRegistry,
    pub events: AudioEventList,
    pub variables: HashMap<String, Value>,
    pub groups: HashMap<String, Vec<Statement>>,
    pub banks: BankRegistry,
    /// Registered global automations
    pub automation_registry: crate::engine::audio::automation::AutomationRegistry,
    /// Per-target note-mode automation contexts (including templates and timing info)
    pub note_automation_templates: std::collections::HashMap<String, NoteAutomationContext>,
    pub cursor_time: f32,
    pub special_vars: SpecialVarContext,
    pub event_registry: EventRegistry,
    #[cfg(feature = "cli")]
    pub midi_manager: Option<std::sync::Arc<std::sync::Mutex<MidiManager>>>,
    /// Track current statement location for better error reporting
    current_statement_location: Option<(usize, usize)>, // (line, column)
    /// Internal guard to avoid re-entrant beat emission during handler execution
    pub suppress_beat_emit: bool,
    /// Internal guard to suppress printing during simulated/local interpreter runs
    pub suppress_print: bool,
    /// Flag used by 'break' statement to request breaking out of loops
    pub break_flag: bool,
    /// Background worker channel sender/receiver (threads send AudioEventList here)
    pub background_event_tx:
        Option<std::sync::mpsc::Sender<crate::engine::audio::events::AudioEventList>>,
    pub background_event_rx:
        Option<std::sync::mpsc::Receiver<crate::engine::audio::events::AudioEventList>>,
    /// Holds join handles for background workers (optional, can be left running)
    pub background_workers: Vec<std::thread::JoinHandle<()>>,
    /// Optional channel used to replay prints in realtime during offline rendering.
    pub realtime_print_tx: Option<std::sync::mpsc::Sender<(f32, String)>>,
    /// Depth of active function calls. Used to validate 'return' usage (only valid inside functions).
    pub function_call_depth: usize,
    /// Function return state: when executing a function, a `return` statement sets
    /// this flag and stores the returned value here so callers can inspect it.
    pub returning_flag: bool,
    pub return_value: Option<Value>,
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
            banks: BankRegistry::new(),
            automation_registry: crate::engine::audio::automation::AutomationRegistry::new(),
            note_automation_templates: std::collections::HashMap::new(),
            cursor_time: 0.0,
            special_vars: SpecialVarContext::new(120.0, sample_rate),
            event_registry: EventRegistry::new(),
            #[cfg(feature = "cli")]
            midi_manager: None,
            current_statement_location: None,
            suppress_beat_emit: false,
            suppress_print: false,
            break_flag: false,
            background_event_tx: None,
            background_event_rx: None,
            background_workers: Vec::new(),
            realtime_print_tx: None,
            function_call_depth: 0,
            returning_flag: false,
            return_value: None,
        }
    }

    /// Handle a trigger statement (e.g., .kit.kick or kit.kick)
    fn handle_trigger(&mut self, entity: &str) -> Result<()> {
        // Delegate detailed trigger handling to the handler module
        handler::handle_trigger(self, entity, None)
    }

    /// Helper to print banks and triggers for debugging
    fn debug_list_banks(&self) {
        // helper intentionally silent in production builds
    }

    pub fn interpret(&mut self, statements: &[Statement]) -> Result<Vec<f32>> {
        // Initialize special vars context
        let total_duration = self.calculate_total_duration(statements)?;
        self.special_vars.total_duration = total_duration;
        self.special_vars.update_bpm(self.bpm);

        // Phase 1: Collect events
        self.collect_events(statements)?;

        // If background 'pass' workers were spawned, drain their produced batches
        // until we've received events covering the estimated total duration. This
        // ensures offline/non-live renders include asynchronous loop-pass events
        // (which are produced by background workers) before rendering audio.
        if self.background_event_rx.is_some() {
            use std::sync::mpsc::RecvTimeoutError;
            use std::time::{Duration, Instant};

            let rx = self.background_event_rx.as_mut().unwrap();
            let start = Instant::now();
            // Hard cap to avoid waiting forever: allow up to total_duration + 5s
            let hard_limit = Duration::from_secs_f32(total_duration + 5.0);
            loop {
                match rx.recv_timeout(Duration::from_millis(200)) {
                    Ok(events) => {
                        self.events.merge(events);
                    }
                    Err(RecvTimeoutError::Timeout) => {
                        // If we've already got events extending to the target duration, stop draining
                        if self.events.total_duration() >= (total_duration - 0.001) {
                            break;
                        }
                        if start.elapsed() > hard_limit {
                            // give up after hard limit
                            break;
                        }
                        // else continue waiting
                    }
                    Err(RecvTimeoutError::Disconnected) => {
                        // Sender(s) dropped; nothing more will arrive
                        break;
                    }
                }
            }
        }

        // Phase 2: Render audio

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
    pub fn calculate_total_duration(&self, _statements: &[Statement]) -> Result<f32> {
        // For now, return a default duration (will be updated during collect_events)
        Ok(60.0) // Default 60 seconds
    }

    pub fn collect_events(&mut self, statements: &[Statement]) -> Result<()> {
        // Delegate to the collector child module
        collector::collect_events(self, statements)
    }
    pub fn handle_let(&mut self, name: &str, value: &Value) -> Result<()> {
        handler::handle_let(self, name, value)
    }

    pub fn extract_audio_event(
        &mut self,
        target: &str,
        context: &crate::engine::functions::FunctionContext,
    ) -> Result<()> {
        // Delegate to extractor child module
        extractor::extract_audio_event(self, target, context)
    }

    pub fn render_audio(&self) -> Result<Vec<f32>> {
        // Delegate to renderer child module
        renderer::render_audio(self)
    }

    pub fn set_bpm(&mut self, bpm: f32) {
        self.bpm = bpm.max(1.0).min(999.0);
        // Keep special vars in sync so $beat/$bar calculations use the updated BPM
        self.special_vars.update_bpm(self.bpm);
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
    pub fn execute_print(&mut self, value: &Value) -> Result<()> {
        handler::execute_print(self, value)
    }

    /// Interpolate variables in a string
    /// Replaces {variable_name} with the variable's value
    pub fn interpolate_string(&self, template: &str) -> String {
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
    pub fn execute_if(
        &mut self,
        condition: &Value,
        body: &[Statement],
        else_body: &Option<Vec<Statement>>,
    ) -> Result<()> {
        handler::execute_if(self, condition, body, else_body)
    }

    /// Evaluate a condition to a boolean
    /// Supports: ==, !=, <, >, <=, >=
    pub fn evaluate_condition(&mut self, condition: &Value) -> Result<bool> {
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
            let left_val = self.resolve_value(left)?;
            let right_val = self.resolve_value(right)?;

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
            // Direct boolean value or identifier: resolve identifiers first
            let resolved = self.resolve_value(condition)?;
            match resolved {
                Value::Boolean(b) => Ok(b),
                Value::Number(n) => Ok(n != 0.0),
                Value::String(s) => Ok(!s.is_empty()),
                Value::Identifier(id) => {
                    // Resolve further if needed (shouldn't usually happen)
                    let further = self.resolve_value(&Value::Identifier(id))?;
                    match further {
                        Value::Boolean(b2) => Ok(b2),
                        Value::Number(n2) => Ok(n2 != 0.0),
                        Value::String(s2) => Ok(!s2.is_empty()),
                        _ => Ok(false),
                    }
                }
                _ => Ok(false),
            }
        }
    }

    /// Resolve a value (replace identifiers with their values from variables)
    pub fn resolve_value(&mut self, value: &Value) -> Result<Value> {
        match value {
            Value::Identifier(name) => {
                // Check if it's a special variable
                if is_special_var(name) {
                    if let Some(special_val) = resolve_special_var(name, &self.special_vars) {
                        return Ok(special_val);
                    }
                }

                // Support combined property/index access like `obj.prop` and `arr[0]` or mixed `arr[0].prop`
                if name.contains('.') || name.contains('[') {
                    // parse segments: start with initial identifier, then .prop or [index]
                    let mut chars = name.chars().peekable();
                    // read initial identifier
                    let mut ident = String::new();
                    while let Some(&c) = chars.peek() {
                        if c == '.' || c == '[' {
                            break;
                        }
                        ident.push(c);
                        chars.next();
                    }

                    let mut current: Option<Value> = if ident.is_empty() {
                        None
                    } else if is_special_var(&ident) {
                        resolve_special_var(&ident, &self.special_vars)
                    } else {
                        self.variables.get(&ident).cloned()
                    };

                    if current.is_none() {
                        return Ok(Value::Null);
                    }

                    // iterate segments
                    while let Some(&c) = chars.peek() {
                        if c == '.' {
                            // consume '.'
                            chars.next();
                            // read property name
                            let mut prop = String::new();
                            while let Some(&nc) = chars.peek() {
                                if nc == '.' || nc == '[' {
                                    break;
                                }
                                prop.push(nc);
                                chars.next();
                            }
                            // descend into map
                            match current {
                                Some(Value::Map(ref map)) => {
                                    current = map.get(&prop).cloned();
                                }
                                _ => {
                                    return Ok(Value::Null);
                                }
                            }
                            if current.is_none() {
                                return Ok(Value::Null);
                            }
                        } else if c == '[' {
                            // consume '['
                            chars.next();
                            // read until ']'
                            let mut idx_tok = String::new();
                            while let Some(&nc) = chars.peek() {
                                if nc == ']' {
                                    break;
                                }
                                idx_tok.push(nc);
                                chars.next();
                            }
                            // consume ']'
                            if let Some(&nc) = chars.peek() {
                                if nc == ']' {
                                    chars.next();
                                } else {
                                    return Ok(Value::Null); // malformed
                                }
                            } else {
                                return Ok(Value::Null); // malformed
                            }

                            let idx_tok_trim = idx_tok.trim();

                            // resolve index token: it can be a number or identifier
                            let index_value = if let Ok(n) = idx_tok_trim.parse::<usize>() {
                                Value::Number(n as f32)
                            } else {
                                Value::Identifier(idx_tok_trim.to_string())
                            };

                            // apply index to current
                            match current {
                                Some(Value::Array(ref arr)) => {
                                    // resolve index value to number. Support simple expressions like
                                    // `i + 1` and post-increment `i++` (treated as i+1 without mutation).
                                    // Evaluate index token. Support mutation for `i++` (post-increment):
                                    // When encountering `ident++`, mutate the variable in-place and use the old value.
                                    let resolved_idx = if idx_tok_trim.ends_with("++") {
                                        // post-increment: mutate var and return old value
                                        let varname = idx_tok_trim[..idx_tok_trim.len() - 2].trim();
                                        let cur = match self.resolve_value(&Value::Identifier(
                                            varname.to_string(),
                                        ))? {
                                            Value::Number(n2) => n2 as isize,
                                            _ => return Ok(Value::Null),
                                        };
                                        self.variables.insert(
                                            varname.to_string(),
                                            Value::Number((cur + 1) as f32),
                                        );
                                        cur
                                    } else if idx_tok_trim.ends_with("--") {
                                        // post-decrement: mutate var and return old value
                                        let varname = idx_tok_trim[..idx_tok_trim.len() - 2].trim();
                                        let cur = match self.resolve_value(&Value::Identifier(
                                            varname.to_string(),
                                        ))? {
                                            Value::Number(n2) => n2 as isize,
                                            _ => return Ok(Value::Null),
                                        };
                                        self.variables.insert(
                                            varname.to_string(),
                                            Value::Number((cur - 1) as f32),
                                        );
                                        cur
                                    } else if idx_tok_trim.contains('+') {
                                        let parts: Vec<&str> =
                                            idx_tok_trim.splitn(2, '+').collect();
                                        let left = parts[0].trim();
                                        let right = parts[1].trim();
                                        let left_val = if let Ok(n) = left.parse::<isize>() {
                                            n
                                        } else {
                                            match self.resolve_value(&Value::Identifier(
                                                left.to_string(),
                                            ))? {
                                                Value::Number(n2) => n2 as isize,
                                                _ => return Ok(Value::Null),
                                            }
                                        };
                                        let right_val = if let Ok(n) = right.parse::<isize>() {
                                            n
                                        } else {
                                            match self.resolve_value(&Value::Identifier(
                                                right.to_string(),
                                            ))? {
                                                Value::Number(n2) => n2 as isize,
                                                _ => return Ok(Value::Null),
                                            }
                                        };
                                        left_val + right_val
                                    } else if idx_tok_trim.contains('*') {
                                        let parts: Vec<&str> =
                                            idx_tok_trim.splitn(2, '*').collect();
                                        let left = parts[0].trim();
                                        let right = parts[1].trim();
                                        let left_val = if let Ok(n) = left.parse::<isize>() {
                                            n
                                        } else {
                                            match self.resolve_value(&Value::Identifier(
                                                left.to_string(),
                                            ))? {
                                                Value::Number(n2) => n2 as isize,
                                                _ => return Ok(Value::Null),
                                            }
                                        };
                                        let right_val = if let Ok(n) = right.parse::<isize>() {
                                            n
                                        } else {
                                            match self.resolve_value(&Value::Identifier(
                                                right.to_string(),
                                            ))? {
                                                Value::Number(n2) => n2 as isize,
                                                _ => return Ok(Value::Null),
                                            }
                                        };
                                        left_val * right_val
                                    } else if idx_tok_trim.contains('/') {
                                        let parts: Vec<&str> =
                                            idx_tok_trim.splitn(2, '/').collect();
                                        let left = parts[0].trim();
                                        let right = parts[1].trim();
                                        let left_val = if let Ok(n) = left.parse::<isize>() {
                                            n
                                        } else {
                                            match self.resolve_value(&Value::Identifier(
                                                left.to_string(),
                                            ))? {
                                                Value::Number(n2) => n2 as isize,
                                                _ => return Ok(Value::Null),
                                            }
                                        };
                                        let right_val = if let Ok(n) = right.parse::<isize>() {
                                            n
                                        } else {
                                            match self.resolve_value(&Value::Identifier(
                                                right.to_string(),
                                            ))? {
                                                Value::Number(n2) => n2 as isize,
                                                _ => return Ok(Value::Null),
                                            }
                                        };
                                        if right_val == 0 {
                                            return Err(anyhow::anyhow!(
                                                "Division by zero in index expression"
                                            ));
                                        }
                                        left_val / right_val
                                    } else if idx_tok_trim.contains('%') {
                                        let parts: Vec<&str> =
                                            idx_tok_trim.splitn(2, '%').collect();
                                        let left = parts[0].trim();
                                        let right = parts[1].trim();
                                        let left_val = if let Ok(n) = left.parse::<isize>() {
                                            n
                                        } else {
                                            match self.resolve_value(&Value::Identifier(
                                                left.to_string(),
                                            ))? {
                                                Value::Number(n2) => n2 as isize,
                                                _ => return Ok(Value::Null),
                                            }
                                        };
                                        let right_val = if let Ok(n) = right.parse::<isize>() {
                                            n
                                        } else {
                                            match self.resolve_value(&Value::Identifier(
                                                right.to_string(),
                                            ))? {
                                                Value::Number(n2) => n2 as isize,
                                                _ => return Ok(Value::Null),
                                            }
                                        };
                                        if right_val == 0 {
                                            return Err(anyhow::anyhow!(
                                                "Modulo by zero in index expression"
                                            ));
                                        }
                                        left_val % right_val
                                    } else if idx_tok_trim.contains('-') {
                                        let parts: Vec<&str> =
                                            idx_tok_trim.splitn(2, '-').collect();
                                        let left = parts[0].trim();
                                        let right = parts[1].trim();
                                        let left_val = if let Ok(n) = left.parse::<isize>() {
                                            n
                                        } else {
                                            match self.resolve_value(&Value::Identifier(
                                                left.to_string(),
                                            ))? {
                                                Value::Number(n2) => n2 as isize,
                                                _ => return Ok(Value::Null),
                                            }
                                        };
                                        let right_val = if let Ok(n) = right.parse::<isize>() {
                                            n
                                        } else {
                                            match self.resolve_value(&Value::Identifier(
                                                right.to_string(),
                                            ))? {
                                                Value::Number(n2) => n2 as isize,
                                                _ => return Ok(Value::Null),
                                            }
                                        };
                                        left_val - right_val
                                    } else {
                                        match index_value {
                                            Value::Number(n) => n as isize,
                                            Value::Identifier(ref id) => match self
                                                .resolve_value(&Value::Identifier(id.clone()))?
                                            {
                                                Value::Number(n2) => n2 as isize,
                                                _ => return Ok(Value::Null),
                                            },
                                            _ => return Ok(Value::Null),
                                        }
                                    };

                                    if resolved_idx < 0 {
                                        return Ok(Value::Null);
                                    }
                                    let ui = resolved_idx as usize;
                                    if ui < arr.len() {
                                        let mut elem = arr[ui].clone();
                                        // If the element is a bare Identifier (like C4) and there is no
                                        // variable with that name, treat it as a String token for convenience
                                        if let Value::Identifier(ref id) = elem {
                                            if !is_special_var(id)
                                                && self.variables.get(id).is_none()
                                            {
                                                elem = Value::String(id.clone());
                                            }
                                        }

                                        // If the element is a Map with a 'value' key, unwrap it so
                                        // myArray[0] returns the inner value directly and so that
                                        // chained access like myArray[2].volume works (we operate on
                                        // the inner value map)
                                        if let Value::Map(ref m) = elem {
                                            if let Some(inner) = m.get("value") {
                                                current = Some(inner.clone());
                                            } else {
                                                current = Some(elem);
                                            }
                                        } else {
                                            current = Some(elem);
                                        }
                                    } else {
                                        return Err(anyhow::anyhow!(
                                            "Index out of range: {} (len={})",
                                            ui,
                                            arr.len()
                                        ));
                                    }
                                }
                                Some(Value::Map(ref map)) => {
                                    // use index token as key
                                    let key = idx_tok_trim.trim_matches('"').trim_matches('\'');
                                    current = map.get(key).cloned();
                                    if current.is_none() {
                                        return Ok(Value::Null);
                                    }
                                }
                                _ => {
                                    return Ok(Value::Null);
                                }
                            }
                        } else {
                            // unexpected char
                            break;
                        }
                    }

                    return Ok(current.unwrap_or(Value::Null));
                }

                // Otherwise, look in variables; if not found, treat bare identifier as a string token
                return Ok(self
                    .variables
                    .get(name)
                    .cloned()
                    .unwrap_or(Value::String(name.clone())));
            }
            Value::Call { name, args } => {
                // Evaluate call expression: resolve args then execute function/group/pattern
                // Resolve each arg
                let mut resolved_args: Vec<Value> = Vec::new();
                for a in args.iter() {
                    let rv = self.resolve_value(a)?;
                    resolved_args.push(rv);
                }

                // Delegate to handler to execute call and return a Value
                let res = super::handler::call_function(self, name, &resolved_args)?;
                return Ok(res);
            }
            _ => return Ok(value.clone()),
        }
    }

    /// Execute event handlers matching the event name
    pub fn execute_event_handlers(&mut self, event_name: &str) -> Result<()> {
        handler::execute_event_handlers(self, event_name)
    }

    /// Check if two values are equal
    pub fn values_equal(&self, left: &Value, right: &Value) -> bool {
        match (left, right) {
            (Value::Number(a), Value::Number(b)) => (a - b).abs() < 0.0001,
            (Value::String(a), Value::String(b)) => a == b,
            (Value::Boolean(a), Value::Boolean(b)) => a == b,
            (Value::Null, Value::Null) => true,
            _ => false,
        }
    }

    /// Compare two values
    pub fn compare_values(
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
    pub fn handle_assign(&mut self, target: &str, property: &str, value: &Value) -> Result<()> {
        handler::handle_assign(self, target, property, value)
    }

    /// Extract synth definition from a map
    pub fn extract_synth_def_from_map(
        &self,
        map: &HashMap<String, Value>,
    ) -> Result<SynthDefinition> {
        handler::extract_synth_def_from_map(self, map)
    }

    /// Handle MIDI file loading: @load "path.mid" as alias
    pub fn handle_load(&mut self, source: &str, alias: &str) -> Result<()> {
        handler::handle_load(self, source, alias)
    }

    /// Handle MIDI binding: bind source -> target { options }
    pub fn handle_bind(&mut self, source: &str, target: &str, options: &Value) -> Result<()> {
        handler::handle_bind(self, source, target, options)
    }

    /// Extract pattern string and options from pattern value
    pub fn extract_pattern_data(
        &self,
        value: &Value,
    ) -> (Option<String>, Option<HashMap<String, f32>>) {
        handler::extract_pattern_data(self, value)
    }

    /// Execute a pattern with given target and pattern string
    pub fn execute_pattern(
        &mut self,
        target: &str,
        pattern: &str,
        options: Option<HashMap<String, f32>>,
    ) -> Result<()> {
        handler::execute_pattern(self, target, pattern, options)
    }

    /// Resolve sample URI from bank.trigger notation (e.g., myBank.kick -> devalang://bank/devaloop.808/kick)
    pub fn resolve_sample_uri(&self, target: &str) -> String {
        handler::resolve_sample_uri(self, target)
    }
}

#[cfg(test)]
#[path = "test_arrays.rs"]
mod tests_arrays;

#[cfg(test)]
#[path = "test_print.rs"]
mod tests_print;

#[cfg(test)]
#[path = "test_functions.rs"]
mod tests_functions;

#[cfg(test)]
#[path = "test_control_flow.rs"]
mod tests_control_flow;
