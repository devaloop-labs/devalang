use crate::{
    config::driver::ProjectConfig,
    core::{
        builder::Builder,
        debugger::{
            lexer::write_lexer_log_file,
            logs::{write_module_function_log_file, write_module_variable_log_file},
            preprocessor::write_preprocessor_log_file,
            store::{write_function_log_file, write_variables_log_file},
        },
        preprocessor::loader::ModuleLoader,
        store::global::GlobalStore,
    },
};
use devalang_utils::path::normalize_path;
use devalang_utils::{
    logger::{LogLevel, Logger},
    spinner::start_spinner,
};

pub fn process_play(
    _config: &Option<ProjectConfig>,
    entry_file: &str,
    output: &str,
    audio_format: crate::cli::parser::AudioFormat,
    sample_rate: u32,
    debug: bool,
) -> Result<
    (
        f32,
        Vec<crate::core::parser::statement::Statement>,
        devalang_types::VariableTable,
        devalang_types::FunctionTable,
        crate::core::store::global::GlobalStore,
    ),
    String,
> {
    let spinner = start_spinner("Building...");

    let normalized_entry = normalize_path(entry_file);
    let normalized_output_dir = normalize_path(output);

    let duration = std::time::Instant::now();
    let mut global_store = GlobalStore::new();
    let loader = ModuleLoader::new(&normalized_entry, &normalized_output_dir);
    let (modules_tokens, modules_statements) = loader.load_all_modules(&mut global_store);

    // Try to detect initial BPM from statements (fallback to 120.0)
    let mut detected_bpm: f32 = 120.0;
    let mut entry_statements: Vec<crate::core::parser::statement::Statement> = Vec::new();
    // Prefer the entry module if present
    if let Some(entry_stmts) = modules_statements.get(&normalized_entry) {
        entry_statements = entry_stmts.clone();
        for stmt in entry_stmts {
            if let crate::core::parser::statement::StatementKind::Tempo = &stmt.kind {
                use devalang_types::Value;
                if let Value::Number(n) = &stmt.value {
                    detected_bpm = *n;
                    break;
                }
            }
        }
    }
    // If still default, scan other modules for a tempo directive
    if (detected_bpm - 120.0).abs() < f32::EPSILON {
        'outer: for (_name, stmts) in modules_statements.iter() {
            for stmt in stmts {
                if let crate::core::parser::statement::StatementKind::Tempo = &stmt.kind {
                    use devalang_types::Value;
                    if let Value::Number(n) = &stmt.value {
                        detected_bpm = *n;
                        break 'outer;
                    }
                }
            }
        }
    }

    // SECTION Write logs
    if debug {
        for (module_path, module) in global_store.modules.clone() {
            write_module_variable_log_file(
                &normalized_output_dir,
                &module_path,
                &module.variable_table,
            );
            write_module_function_log_file(
                &normalized_output_dir,
                &module_path,
                &module.function_table,
            );
        }

        write_lexer_log_file(
            &normalized_output_dir,
            "lexer_tokens.log",
            modules_tokens.clone(),
        );
        write_preprocessor_log_file(
            &normalized_output_dir,
            "resolved_statements.log",
            modules_statements.clone(),
        );
        write_variables_log_file(
            &normalized_output_dir,
            "global_variables.log",
            global_store.variables.clone(),
        );
        write_function_log_file(
            &normalized_output_dir,
            "global_functions.log",
            global_store.functions.clone(),
        );
    }

    // SECTION Detect errors before building (like build.rs)
    let all_errors = crate::core::error::collect_all_errors_with_modules(&modules_statements);
    let (warnings, criticals) = crate::core::error::partition_errors(all_errors);
    crate::core::error::log_errors_with_stack("Play", &warnings, &criticals);
    if !criticals.is_empty() {
        spinner.finish_and_clear();
        return Err(format!(
            "play failed with {} critical error(s): {}",
            criticals.len(),
            criticals[0].message
        ));
    }

    // SECTION Building AST and Audio
    let builder = Builder::new();
    builder.build_ast(&modules_statements, output, false);
    let audio_format_str = format!("{:?}", audio_format);
    builder.build_audio(
        &modules_statements,
        output,
        &mut global_store,
        Some(audio_format_str),
        Some(sample_rate),
    );

    // SECTION Logging
    let logger = Logger::new();
    let success_message = format!(
        "Build completed successfully in {:.2?}. Output files written to: '{}'",
        duration.elapsed(),
        normalized_output_dir
    );

    spinner.finish_and_clear();
    logger.log_message(LogLevel::Success, &success_message);

    Ok((
        detected_bpm,
        entry_statements,
        global_store.variables.clone(),
        global_store.functions.clone(),
        global_store,
    ))
}
