pub mod driver;
pub mod export;
pub mod helpers;
pub mod notes;
pub mod sample;

pub use driver::AudioEngine;

pub use driver::CHANNELS;
pub use driver::MidiNoteEvent;
pub use driver::SAMPLE_RATE;
pub use helpers::*;

use crate::core::audio::interpreter::driver::run_audio_program;
use crate::core::parser::statement::Statement;
use crate::core::store::global::GlobalStore;
use devalang_types::{FunctionTable, VariableTable};
use std::collections::HashMap;

/// Render audio for a set of parsed modules.
///
/// For each entry in `modules` (module path -> statements), create an
/// AudioEngine, setup empty module-local VariableTable and FunctionTable,
/// invoke the interpreter driver to schedule audio into the engine, and
/// return a map of module name -> AudioEngine.
pub fn render_audio_with_modules(
    modules: HashMap<String, Vec<Statement>>,
    _output_dir: &str,
    global_store: &mut GlobalStore,
) -> HashMap<String, AudioEngine> {
    let mut result: HashMap<String, AudioEngine> = HashMap::new();

    for (module_name, statements) in modules.into_iter() {
        // Create engine named after module (used for diagnostics)
        let mut engine = AudioEngine::new(module_name.clone());

        // Create empty module-local tables; interpreter expects these as starting context
        let module_vars = VariableTable::new();
        let module_funcs = FunctionTable::new();

        // Run interpreter which will populate the engine.buffer and midi events
        let (_max_end_time, _cursor_time) = run_audio_program(
            &statements,
            &mut engine,
            module_name.clone(),
            module_name.clone(),
            module_vars,
            module_funcs,
            global_store,
        );

        result.insert(module_name, engine);
    }

    result
}
