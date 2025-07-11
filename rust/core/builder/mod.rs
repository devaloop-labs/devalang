use crate::core::audio::render::render_audio_with_modules;
use crate::core::parser::statement::Statement;
use crate::core::store::global::GlobalStore;
use std::{ collections::HashMap, fs::create_dir_all };
use std::io::Write;

use crate::utils::logger::Logger;

pub struct Builder {}

impl Builder {
    pub fn new() -> Self {
        Builder {}
    }

    pub fn build_ast(&self, modules: &HashMap<String, Vec<Statement>>, out_dir: &str) {
        for (name, statements) in modules {
            let formatted_name = name.split("/").last().unwrap_or(name);
            let formatted_name = formatted_name.replace(".deva", "");

            create_dir_all(format!("{}/ast", out_dir)).expect("Failed to create AST directory");

            let file_path = format!("{}/ast/{}.json", out_dir, formatted_name);
            let mut file = std::fs::File::create(file_path).expect("Failed to create AST file");

            let content = serde_json
                ::to_string_pretty(&statements)
                .expect("Failed to serialize AST");

            file.write_all(content.as_bytes()).expect("Failed to write AST to file");
        }
    }

    pub fn build_audio(
        &self,
        modules: &HashMap<String, Vec<Statement>>,
        normalized_output_dir: &str,
        global_store: &mut GlobalStore
    ) {
        let logger = Logger::new();

        let audio_engines = render_audio_with_modules(
            modules.clone(),
            &normalized_output_dir,
            global_store
        );

        create_dir_all(format!("{}/audio", normalized_output_dir)).expect(
            "Failed to create audio directory"
        );

        for (module_name, mut audio_engine) in audio_engines {
            let formatted_module_name = module_name
                .split('/')
                .last()
                .unwrap_or(&module_name)
                .replace(".deva", "");

            let output_path = format!(
                "{}/audio/{}.wav",
                normalized_output_dir,
                formatted_module_name
            );

            match audio_engine.generate_wav_file(&output_path) {
                Ok(_) => {}
                Err(msg) => {
                    logger.log_error_with_stacktrace(
                        &format!(
                            "Unable to generate WAV file for module '{}': {}",
                            formatted_module_name,
                            msg
                        ),
                        &module_name
                    );
                }
            }
        }
    }
}
