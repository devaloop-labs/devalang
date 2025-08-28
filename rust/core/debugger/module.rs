use crate::core::{
    debugger::Debugger,
    store::{function::FunctionTable, variable::VariableTable},
};
use std::fs::create_dir_all;

pub fn write_module_variable_log_file(
    output_dir: &str,
    module_path: &str,
    variable_table: &VariableTable,
) {
    let debugger = Debugger::new();
    let mut content = String::new();
    let module_name = module_path
        .split('/')
        .last()
        .unwrap_or("index")
        .replace(".deva", "");

    let log_directory = format!("{}/logs/modules/{}", output_dir, module_name);
    create_dir_all(&log_directory).expect("Failed to create log directory");

    for (var_name, var_data) in &variable_table.variables {
        content.push_str(&format!("{:?} = {:?}\n", var_name, var_data));
    }

    content.push_str("\n");

    debugger.write_log_file(&log_directory, "variables.log", &content);
}

pub fn write_module_function_log_file(
    output_dir: &str,
    module_path: &str,
    function_table: &FunctionTable,
) {
    let debugger = Debugger::new();
    let mut content = String::new();
    let module_name = module_path
        .split('/')
        .last()
        .unwrap_or("index")
        .replace(".deva", "");

    let log_directory = format!("{}/logs/modules/{}", output_dir, module_name);
    create_dir_all(&log_directory).expect("Failed to create log directory");

    for (func_name, func_data) in &function_table.functions {
        content.push_str(&format!("{:?} = {:?}\n", func_name, func_data));
    }

    content.push_str("\n");

    debugger.write_log_file(&log_directory, "functions.log", &content);
}
