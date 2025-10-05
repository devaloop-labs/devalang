/// Special variables system for Devalang
/// Provides runtime-computed variables like $beat, $time, $random, etc.
use crate::language::syntax::ast::Value;
use std::collections::HashMap;

/// Special variable prefix
pub const SPECIAL_VAR_PREFIX: char = '$';

/// Special variable categories
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpecialVarCategory {
    Time,     // $time, $beat, $bar
    Random,   // $random.noise, $random.float, $random.int
    Music,    // $bpm, $tempo, $duration
    Position, // $position, $progress
    Midi,     // $midi.note, $midi.velocity
    System,   // $sampleRate, $channels
}

/// Special variable context - holds runtime state
#[derive(Debug, Clone)]
pub struct SpecialVarContext {
    pub current_time: f32,   // Current playback time in seconds
    pub current_beat: f32,   // Current beat position
    pub current_bar: f32,    // Current bar position
    pub bpm: f32,            // Current BPM
    pub duration: f32,       // Beat duration in seconds
    pub sample_rate: u32,    // Sample rate
    pub channels: usize,     // Number of channels
    pub position: f32,       // Normalized position (0.0-1.0)
    pub total_duration: f32, // Total duration in seconds
}

impl Default for SpecialVarContext {
    fn default() -> Self {
        Self {
            current_time: 0.0,
            current_beat: 0.0,
            current_bar: 0.0,
            bpm: 120.0,
            duration: 0.5,
            sample_rate: 44100,
            channels: 2,
            position: 0.0,
            total_duration: 0.0,
        }
    }
}

impl SpecialVarContext {
    pub fn new(bpm: f32, sample_rate: u32) -> Self {
        Self {
            bpm,
            duration: 60.0 / bpm,
            sample_rate,
            ..Default::default()
        }
    }

    /// Update time-based variables
    pub fn update_time(&mut self, time: f32) {
        self.current_time = time;
        self.current_beat = time / self.duration;
        self.current_bar = self.current_beat / 4.0;

        if self.total_duration > 0.0 {
            self.position = (time / self.total_duration).clamp(0.0, 1.0);
        }
    }

    /// Update BPM
    pub fn update_bpm(&mut self, bpm: f32) {
        self.bpm = bpm;
        self.duration = 60.0 / bpm;
    }
}

/// Check if a variable name is a special variable
pub fn is_special_var(name: &str) -> bool {
    name.starts_with(SPECIAL_VAR_PREFIX)
}

/// Resolve a special variable to its current value
pub fn resolve_special_var(name: &str, context: &SpecialVarContext) -> Option<Value> {
    if !is_special_var(name) {
        return None;
    }

    match name {
        // Time variables
        "$time" => Some(Value::Number(context.current_time)),
        "$beat" => Some(Value::Number(context.current_beat)),
        "$bar" => Some(Value::Number(context.current_bar)),
        "$currentTime" => Some(Value::Number(context.current_time)),
        "$currentBeat" => Some(Value::Number(context.current_beat)),
        "$currentBar" => Some(Value::Number(context.current_bar)),

        // Music variables
        "$bpm" => Some(Value::Number(context.bpm)),
        "$tempo" => Some(Value::Number(context.bpm)),
        "$duration" => Some(Value::Number(context.duration)),

        // Position variables
        "$position" => Some(Value::Number(context.position)),
        "$progress" => Some(Value::Number(context.position)),

        // System variables
        "$sampleRate" => Some(Value::Number(context.sample_rate as f32)),
        "$channels" => Some(Value::Number(context.channels as f32)),

        // Random variables (computed on-demand)
        "$random" | "$random.float" => Some(Value::Number(rand::random::<f32>())),
        "$random.noise" => Some(Value::Number(rand::random::<f32>() * 2.0 - 1.0)), // -1.0 to 1.0
        "$random.int" => Some(Value::Number((rand::random::<u32>() % 100) as f32)),
        "$random.bool" => Some(Value::Boolean(rand::random::<bool>())),

        // Nested random with ranges
        _ if name.starts_with("$random.range(") => {
            // Parse $random.range(min, max)
            parse_random_range(name)
        }

        _ => None,
    }
}

/// Parse $random.range(min, max) syntax
fn parse_random_range(name: &str) -> Option<Value> {
    // Extract content between parentheses
    let start = name.find('(')?;
    let end = name.rfind(')')?;
    let content = &name[start + 1..end];

    // Split by comma
    let parts: Vec<&str> = content.split(',').map(|s| s.trim()).collect();
    if parts.len() != 2 {
        return None;
    }

    // Parse min and max
    let min: f32 = parts[0].parse().ok()?;
    let max: f32 = parts[1].parse().ok()?;

    // Generate random value in range
    let value = min + rand::random::<f32>() * (max - min);
    Some(Value::Number(value))
}

/// Get all available special variables as a map
pub fn get_all_special_vars(context: &SpecialVarContext) -> HashMap<String, Value> {
    let mut vars = HashMap::new();

    // Time
    vars.insert("$time".to_string(), Value::Number(context.current_time));
    vars.insert("$beat".to_string(), Value::Number(context.current_beat));
    vars.insert("$bar".to_string(), Value::Number(context.current_bar));
    vars.insert(
        "$currentTime".to_string(),
        Value::Number(context.current_time),
    );
    vars.insert(
        "$currentBeat".to_string(),
        Value::Number(context.current_beat),
    );
    vars.insert(
        "$currentBar".to_string(),
        Value::Number(context.current_bar),
    );

    // Music
    vars.insert("$bpm".to_string(), Value::Number(context.bpm));
    vars.insert("$tempo".to_string(), Value::Number(context.bpm));
    vars.insert("$duration".to_string(), Value::Number(context.duration));

    // Position
    vars.insert("$position".to_string(), Value::Number(context.position));
    vars.insert("$progress".to_string(), Value::Number(context.position));

    // System
    vars.insert(
        "$sampleRate".to_string(),
        Value::Number(context.sample_rate as f32),
    );
    vars.insert(
        "$channels".to_string(),
        Value::Number(context.channels as f32),
    );

    vars
}

/// List all special variable categories with examples
pub fn list_special_vars() -> HashMap<&'static str, Vec<(&'static str, &'static str)>> {
    let mut categories = HashMap::new();

    categories.insert(
        "Time",
        vec![
            ("$time", "Current time in seconds"),
            ("$beat", "Current beat position"),
            ("$bar", "Current bar position"),
            ("$currentTime", "Alias for $time"),
            ("$currentBeat", "Alias for $beat"),
            ("$currentBar", "Alias for $bar"),
        ],
    );

    categories.insert(
        "Music",
        vec![
            ("$bpm", "Current BPM"),
            ("$tempo", "Alias for $bpm"),
            ("$duration", "Beat duration in seconds"),
        ],
    );

    categories.insert(
        "Position",
        vec![
            ("$position", "Normalized position (0.0-1.0)"),
            ("$progress", "Alias for $position"),
        ],
    );

    categories.insert(
        "Random",
        vec![
            ("$random", "Random float 0.0-1.0"),
            ("$random.float", "Random float 0.0-1.0"),
            ("$random.noise", "Random float -1.0 to 1.0"),
            ("$random.int", "Random integer 0-99"),
            ("$random.bool", "Random boolean"),
            ("$random.range(min, max)", "Random float in range"),
        ],
    );

    categories.insert(
        "System",
        vec![
            ("$sampleRate", "Sample rate in Hz"),
            ("$channels", "Number of audio channels"),
        ],
    );

    categories
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_special_var() {
        assert!(is_special_var("$time"));
        assert!(is_special_var("$beat"));
        assert!(is_special_var("$random"));
        assert!(!is_special_var("time"));
        assert!(!is_special_var("myVar"));
    }

    #[test]
    fn test_resolve_time_vars() {
        let mut context = SpecialVarContext::default();
        context.update_time(2.0);

        let time = resolve_special_var("$time", &context);
        assert_eq!(time, Some(Value::Number(2.0)));

        let beat = resolve_special_var("$beat", &context);
        assert!(matches!(beat, Some(Value::Number(_))));
    }

    #[test]
    fn test_resolve_random_vars() {
        let context = SpecialVarContext::default();

        let rand1 = resolve_special_var("$random", &context);
        assert!(matches!(rand1, Some(Value::Number(_))));

        let rand2 = resolve_special_var("$random.noise", &context);
        assert!(matches!(rand2, Some(Value::Number(_))));
    }

    #[test]
    fn test_context_update_time() {
        let mut context = SpecialVarContext::new(120.0, 44100);
        context.update_time(1.0);

        assert_eq!(context.current_time, 1.0);
        assert!(context.current_beat > 0.0);
    }

    #[test]
    fn test_parse_random_range() {
        let result = parse_random_range("$random.range(0, 10)");
        assert!(result.is_some());

        if let Some(Value::Number(n)) = result {
            assert!(n >= 0.0 && n <= 10.0);
        }
    }
}
