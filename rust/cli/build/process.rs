use crate::core::{
    builder::Builder,
    debugger::{
        lexer::write_lexer_log_file,
        module::{write_module_function_log_file, write_module_variable_log_file},
        preprocessor::write_preprocessor_log_file,
        store::{write_function_log_file, write_variables_log_file},
    },
    preprocessor::loader::ModuleLoader,
    store::global::GlobalStore,
    utils::path::normalize_path,
};
use devalang_utils::{
    logger::{LogLevel, Logger},
    spinner::start_spinner,
};

pub struct BuildStatsInput {
    pub statements_by_module:
        std::collections::HashMap<String, Vec<crate::core::parser::statement::Statement>>,
    pub global_store: crate::core::store::global::GlobalStore,
}

pub fn process_build(
    entry: String,
    output: String,
    debug: bool,
    compress: bool,
) -> Result<BuildStatsInput, String> {
    let spinner = start_spinner("Building...");

    let duration = std::time::Instant::now();

    let normalized_entry_file = normalize_path(&entry);
    let normalized_output_dir = normalize_path(&output);

    let mut global_store = GlobalStore::new();
    let module_loader = ModuleLoader::new(&normalized_entry_file, &normalized_output_dir);

    // SECTION Load
    // NOTE: We use modules in the build command, so we need to load them
    let (modules_tokens, modules_statements) = module_loader.load_all_modules(&mut global_store);

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

    // SECTION Detect build-time errors prior to building
    let all_errors = crate::core::error::collect_all_errors_with_modules(&modules_statements);
    let (warnings, criticals) = crate::core::error::partition_errors(all_errors);
    crate::core::error::log_errors_with_stack("Build", &warnings, &criticals);
    if !criticals.is_empty() {
        spinner.finish_and_clear();
        return Err(format!(
            "build failed with {} critical error(s): {}",
            criticals.len(),
            criticals[0].message
        ));
    }

    // SECTION Building AST and Audio
    let builder = Builder::new();
    builder.build_ast(&modules_statements, &normalized_output_dir, compress);
    builder.build_audio(
        &modules_statements,
        &normalized_output_dir,
        &mut global_store,
    );

    // SECTION Logging
    let logger = Logger::new();

    if debug {
        let modules_loaded = global_store.modules.keys().collect::<Vec<_>>();
        let global_variables_loaded = global_store.variables.variables.keys().collect::<Vec<_>>();
        let global_functions_loaded = global_store.functions.functions.keys().collect::<Vec<_>>();

        logger.log_message_with_trace(
            LogLevel::Debug,
            &format!("Modules loaded: {}", global_store.modules.len()),
            modules_loaded.iter().map(|s| s.as_str()).collect(),
        );
        logger.log_message_with_trace(
            LogLevel::Debug,
            &format!(
                "Global variables: {}",
                global_store.variables.variables.len()
            ),
            global_variables_loaded.iter().map(|s| s.as_str()).collect(),
        );
        logger.log_message_with_trace(
            LogLevel::Debug,
            &format!(
                "Global functions: {}",
                global_store.functions.functions.len()
            ),
            global_functions_loaded.iter().map(|s| s.as_str()).collect(),
        );
    }

    let success_message = format!(
        "Build completed successfully in {:.2?}. Output files written to: '{}'",
        duration.elapsed(),
        normalized_output_dir
    );

    spinner.finish_and_clear();
    logger.log_message(LogLevel::Success, &success_message);
    Ok(BuildStatsInput {
        statements_by_module: modules_statements,
        global_store,
    })
}
