use std::{ fs::create_dir_all };
use crate::core::{
    debugger::Debugger,
    store::{ function::FunctionTable, variable::VariableTable },
};

pub fn write_variables_log_file(output_dir: &str, file_name: &str, variables: VariableTable) {
    let debugger = Debugger::new();
    let mut content = String::new();

    let log_directory = format!("{}/logs", output_dir);
    create_dir_all(&log_directory).expect("Failed to create log directory");

    for (var_name, var_data) in variables.variables {
        content.push_str(&format!("{:?} = {:?}\n", var_name, var_data));
    }

    content.push_str("\n");

    debugger.write_log_file(&log_directory, file_name, &content);
}

pub fn write_function_log_file(output_dir: &str, file_name: &str, functions: FunctionTable) {
    let debugger = Debugger::new();
    let mut content = String::new();

    let log_directory = format!("{}/logs", output_dir);
    create_dir_all(&log_directory).expect("Failed to create log directory");

    for (index, function) in functions.functions {
        content.push_str(
            &format!("'{}' = [{:?}] => {:?}\n", function.name, function.parameters, function.body)
        );
    }

    content.push_str("\n");

    debugger.write_log_file(&log_directory, file_name, &content);
}
