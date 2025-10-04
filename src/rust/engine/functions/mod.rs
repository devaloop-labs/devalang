pub mod chord;
pub mod effects;
/// Function execution system for arrow calls
///
/// This module provides a modular system for executing chainable functions
/// like `synth -> note(C4) -> filter(lowpass, 1000)`
pub mod note;

use crate::language::syntax::ast::nodes::Value;
use anyhow::Result;
use std::collections::HashMap;

/// Context passed through function chain
#[derive(Debug, Clone)]
pub struct FunctionContext {
    /// Target object (e.g., synth name, sample name)
    pub target: String,

    /// Accumulated state (e.g., MIDI notes, audio buffer)
    pub state: HashMap<String, Value>,

    /// Timing information
    pub start_time: f32,
    pub duration: f32,

    /// Tempo for beat calculations
    pub tempo: f32,
}

impl FunctionContext {
    pub fn new(target: String, start_time: f32, tempo: f32) -> Self {
        Self {
            target,
            state: HashMap::new(),
            start_time,
            duration: 0.0,
            tempo,
        }
    }

    pub fn set(&mut self, key: impl Into<String>, value: Value) {
        self.state.insert(key.into(), value);
    }

    pub fn get(&self, key: &str) -> Option<&Value> {
        self.state.get(key)
    }
}

/// Trait for executable functions
pub trait FunctionExecutor {
    /// Execute the function with given arguments
    fn execute(&self, context: &mut FunctionContext, args: &[Value]) -> Result<()>;

    /// Function name
    fn name(&self) -> &str;
}

/// Function registry
pub struct FunctionRegistry {
    functions: HashMap<String, Box<dyn FunctionExecutor>>,
}

impl FunctionRegistry {
    pub fn new() -> Self {
        let mut registry = Self {
            functions: HashMap::new(),
        };

        // Register built-in functions
        registry.register(Box::new(note::NoteFunction));
        registry.register(Box::new(chord::ChordFunction));
        registry.register(Box::new(effects::VelocityFunction));
        registry.register(Box::new(effects::DurationFunction));
        registry.register(Box::new(effects::PanFunction));
        registry.register(Box::new(effects::DetuneFunction));
        registry.register(Box::new(effects::SpreadFunction));
        registry.register(Box::new(effects::GainFunction));
        registry.register(Box::new(effects::AttackFunction));
        registry.register(Box::new(effects::ReleaseFunction));
        registry.register(Box::new(effects::DelayFunction));
        registry.register(Box::new(effects::ReverbFunction));
        registry.register(Box::new(effects::DriveFunction));
        registry.register(Box::new(effects::ChorusFunction));
        registry.register(Box::new(effects::FlangerFunction));
        registry.register(Box::new(effects::PhaserFunction));
        registry.register(Box::new(effects::CompressorFunction));
        registry.register(Box::new(effects::DistortionFunction));

        registry
    }

    pub fn register(&mut self, executor: Box<dyn FunctionExecutor>) {
        self.functions.insert(executor.name().to_string(), executor);
    }

    pub fn execute(&self, name: &str, context: &mut FunctionContext, args: &[Value]) -> Result<()> {
        if let Some(func) = self.functions.get(name) {
            func.execute(context, args)
        } else {
            Err(anyhow::anyhow!("Unknown function: {}", name))
        }
    }

    pub fn has(&self, name: &str) -> bool {
        self.functions.contains_key(name)
    }
}

impl Default for FunctionRegistry {
    fn default() -> Self {
        Self::new()
    }
}
