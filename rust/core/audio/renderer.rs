use crate::core::{
    audio::{engine::AudioEngine, interpreter::driver::run_audio_program},
    parser::statement::Statement,
    store::global::GlobalStore,
};
use devalang_utils::logger::{LogLevel, Logger};
use std::collections::HashMap;

pub fn render_audio_with_modules(
    modules: HashMap<String, Vec<Statement>>,
    output_dir: &str,
    global_store: &mut GlobalStore,
) -> HashMap<String, AudioEngine> {
    let mut result = HashMap::new();

    for (module_name, statements) in modules {
        let mut global_max_end_time: f32 = 0.0;
        let mut audio_engine = AudioEngine::new(module_name.clone());

        // Apply global variables to the initial engine
        if let Some(module) = global_store.get_module(&module_name) {
            // interprete statements to fill the audio buffer
            let (module_max_end_time, _cursor_time) = run_audio_program(
                &statements,
                &mut audio_engine,
                module_name.clone(),
                output_dir.to_string(),
                module.variable_table.clone(),
                module.function_table.clone(),
                global_store,
            );

            // Verify if the buffer is silent (all samples are zero)
            if audio_engine.buffer.iter().all(|&s| s == 0) {
                let logger = Logger::new();
                logger.log_message(
                    LogLevel::Warning,
                    &format!(
                        "Module '{}' ignored: silent buffer (no non-zero samples)",
                        module_name
                    ),
                );
            }

            // Determines the maximum end time for the module
            global_max_end_time = global_max_end_time.max(module_max_end_time);
            audio_engine.set_duration(global_max_end_time);

            result.insert(module_name, audio_engine);
        }
    }

    result
}
