use crate::engine::audio::events::AudioEventList;
use crate::engine::events::EventRegistry;
use crate::engine::functions::FunctionRegistry;
use crate::engine::special_vars::{SpecialVarContext, is_special_var, resolve_special_var};
use crate::engine::audio::events::SynthDefinition;
use crate::language::syntax::ast::{Statement, StatementKind, Value};
use crate::language::addons::registry::BankRegistry;
/// Audio interpreter driver - main execution loop
use anyhow::Result;
use std::collections::HashMap;

pub mod collector;
pub mod handler;
pub mod extractor;
pub mod renderer;

pub struct AudioInterpreter {
    pub sample_rate: u32,
    pub bpm: f32,
    pub function_registry: FunctionRegistry,
    pub events: AudioEventList,
    pub variables: HashMap<String, Value>,
    pub groups: HashMap<String, Vec<Statement>>,
    pub banks: BankRegistry,
    pub cursor_time: f32,
    pub special_vars: SpecialVarContext,
    pub event_registry: EventRegistry,
    /// Track current statement location for better error reporting
    current_statement_location: Option<(usize, usize)>, // (line, column)
    /// Internal guard to avoid re-entrant beat emission during handler execution
    pub suppress_beat_emit: bool,
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
            cursor_time: 0.0,
            special_vars: SpecialVarContext::new(120.0, sample_rate),
            event_registry: EventRegistry::new(),
            current_statement_location: None,
            suppress_beat_emit: false,
        }
    }

    /// Handle a trigger statement (e.g., .kit.kick or kit.kick)
    fn handle_trigger(&mut self, entity: &str) -> Result<()> {
        // Delegate detailed trigger handling to the handler module
        handler::handle_trigger(self, entity)
    }

    /// Helper to print banks and triggers for debugging
    fn debug_list_banks(&self) {
        println!("🔍 Déclencheurs disponibles dans BankRegistry:");
        for (bank_name, bank) in self.banks.list_banks() {
            println!("   Banque: {}", bank_name);
            for trigger in bank.list_triggers() {
                println!("      Déclencheur: {}", trigger);
            }
        }
    }

    pub fn interpret(&mut self, statements: &[Statement]) -> Result<Vec<f32>> {
        // Initialize special vars context
        let total_duration = self.calculate_total_duration(statements)?;
        self.special_vars.total_duration = total_duration;
        self.special_vars.update_bpm(self.bpm);

        // Phase 1: Collect events
        self.collect_events(statements)?;

        // Diagnostic: detailed listing of planned audio events (notes/chords/samples)
        {
            use crate::engine::audio::events::AudioEvent;

            // Build a sortable list of (start, end, idx, &AudioEvent)
            let mut list: Vec<(f32, f32, usize, &AudioEvent)> = Vec::new();
            for (i, ev) in self.events.events.iter().enumerate() {
                match ev {
                    AudioEvent::Note { start_time, duration, .. } => {
                        let end = start_time + duration;
                        list.push((*start_time, end, i, ev));
                    }
                    AudioEvent::Chord { start_time, duration, .. } => {
                        let end = start_time + duration;
                        list.push((*start_time, end, i, ev));
                    }
                    AudioEvent::Sample { start_time, .. } => {
                        // estimate sample duration if unknown (used for overlap detection)
                        let end = start_time + 2.0;
                        list.push((*start_time, end, i, ev));
                    }
                }
            }

            list.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));

            println!("🔎 Planned audio events: count={} ", list.len());
            for (s, e, idx, ev) in &list {
                match ev {
                    AudioEvent::Note { midi, start_time, duration, synth_id, .. } => {
                        println!("   [{}] Note midi={} synth='{}' start={:.3}s end={:.3}s dur={:.3}s", idx, midi, synth_id, start_time, e, duration);
                    }
                    AudioEvent::Chord { midis, start_time, duration, synth_id, .. } => {
                        println!("   [{}] Chord midis={:?} synth='{}' start={:.3}s end={:.3}s dur={:.3}s", idx, midis, synth_id, start_time, e, duration);
                    }
                    AudioEvent::Sample { uri, start_time, velocity } => {
                        println!("   [{}] Sample uri='{}' start={:.3}s end={:.3}s vel={:.3}", idx, uri, start_time, e, velocity);
                    }
                }
            }

            // Detect temporal overlaps (any two events whose intervals intersect)
            for i in 0..list.len() {
                let (s1, e1, idx1, ev1) = list[i];
                for j in i+1..list.len() {
                    let (s2, e2, idx2, ev2) = list[j];
                    // if next event starts after current end, break inner loop (list sorted by start)
                    if s2 >= e1 { break; }
                    // otherwise intervals overlap
                    let overlap = (e1.min(e2) - s2).max(0.0);
                    if overlap > 0.0001 {
                        // print summary of overlapping pair
                        println!("⚠️  Overlap detected: idx{} ({:.3}-{:.3}s) ↔ idx{} ({:.3}-{:.3}s) overlap={:.3}s",
                            idx1, s1, e1, idx2, s2, e2, overlap);
                    }
                }
            }
        }

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
    pub fn execute_print(&self, value: &Value) -> Result<()> {
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
    pub fn evaluate_condition(&self, condition: &Value) -> Result<bool> {
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
    pub fn resolve_value(&self, value: &Value) -> Value {
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
    pub fn extract_synth_def_from_map(&self, map: &HashMap<String, Value>) -> Result<SynthDefinition> {
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
    pub fn extract_pattern_data(&self, value: &Value) -> (Option<String>, Option<HashMap<String, f32>>) {
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
