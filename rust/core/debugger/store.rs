use std::{ collections::HashMap, fs::create_dir_all };
use crate::core::{ debugger::Debugger, preprocessor::module::Module };

pub fn write_store_log_file(output_dir: &str, file_name: &str, modules: HashMap<String, Module>) {
    let debugger = Debugger::new();
    let mut content = String::new();

    let log_directory = format!("{}/logs", output_dir);
    create_dir_all(&log_directory).expect("Failed to create log directory");

    for (path, module) in modules {
        content.push_str(&format!("--- Module: {} ---\n", path));

        for (index, var_value) in module.variable_table.variables.iter().enumerate() {
            let var_name = var_value.0.clone();
            let var_data = var_value.1;

            content.push_str(&format!("{}: {:?} = {:?}\n", index + 1, var_name, var_data));
        }

        content.push_str("\n");
    }

    debugger.write_log_file(&log_directory, file_name, &content);
}
