use std::collections::HashMap;

use crate::{
    core::{
        audio::{ engine::AudioEngine, interpreter::driver::run_audio_program },
        parser::statement::Statement,
        store::global::GlobalStore,
    },
    utils::logger::{ LogLevel, Logger },
};

pub fn render_audio_with_modules(
    modules: HashMap<String, Vec<Statement>>,
    output_dir: &str,
    global_store: &mut GlobalStore
) -> HashMap<String, AudioEngine> {
    let mut result = HashMap::new();

    for (module_name, statements) in modules {
        let mut global_max_end_time: f32 = 0.0;
        let mut initial_engine = AudioEngine::new(module_name.clone());

        // Apply global variables to the initial engine
        if let Some(module) = global_store.get_module(&module_name) {
            initial_engine.set_variables(module.variable_table.clone());
        }

        // interprete statements to fill the audio buffer
        let (mut updated_engine, _bpm, module_max_end_time) = run_audio_program(
            &statements,
            initial_engine,
            module_name.clone(),
            output_dir.to_string()
        );

        // Verify if the buffer is silent (all samples are zero)
        if updated_engine.buffer.iter().all(|&s| s == 0) {
            let logger = Logger::new();
            logger.log_message(
                LogLevel::Warning,
                &format!("Module '{}' ignored: silent buffer (no non-zero samples)", module_name)
            );
            continue;
        }

        // Determines the maximum end time for the module
        global_max_end_time = global_max_end_time.max(module_max_end_time);
        updated_engine.set_duration(global_max_end_time);

        result.insert(module_name, updated_engine);
    }

    result
}
