use std::collections::HashMap;

use crate::{
    core::{ audio::{engine::AudioEngine, interpreter::interprete_statements}, parser::statement::Statement, store::global::GlobalStore },
    utils::logger::{ LogLevel, Logger },
};

pub fn render_audio_with_modules(
    modules: HashMap<String, Vec<Statement>>,
    output_dir: &str,
    global_store: &mut GlobalStore
) -> HashMap<String, AudioEngine> {
    let mut result = HashMap::new();

    for (module_name, statements) in modules {
        let mut global_max_end_time = 0.0;
        let mut audio_engine = AudioEngine::new();

        // Apply the module's variable table if it exists
        if let Some(module) = global_store.get_module(&module_name) {
            audio_engine.set_variables(module.variable_table.clone());
        }

        // Interpret the statements to fill the audio buffer
        let (mut audio_engine, module_base_bpm, module_max_end_time) = interprete_statements(
            &statements,
            audio_engine,
            module_name.clone(),
            output_dir.to_string()
        );

        // Calculate the module's maximum duration
        global_max_end_time = module_max_end_time.max(global_max_end_time);
        audio_engine.set_duration(global_max_end_time);

        // Check if the buffer contains at least one non-zero sample
        if audio_engine.buffer.iter().all(|&s| s == 0) {
            let logger = Logger::new();

            logger.log_message(
                LogLevel::Warning,
                format!("Module '{}' ignored: silent buffer (no non-zero samples)", module_name).as_str()
            );

            continue;
        }

        // Insert only if the module produces sound
        result.insert(module_name, audio_engine);
    }

    result
}
