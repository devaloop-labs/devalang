use std::collections::HashMap;

use crate::{
    core::{
        parser::statement::{ Statement, StatementKind },
        preprocessor::{ module::Module, resolver::driver::resolve_statement },
        shared::value::Value,
        store::{ global::GlobalStore, variable::VariableTable },
    },
    utils::logger::{ LogLevel, Logger },
};

fn find_export_value(name: &str, global_store: &GlobalStore) -> Option<Value> {
    for (_path, module) in &global_store.modules {
        if let Some(val) = module.export_table.get_export(name) {
            return Some(val.clone());
        }
    }

    None
}

pub fn resolve_value(value: &Value, module: &Module, global_store: &mut GlobalStore) -> Value {
    match value {
        Value::String(s) => {
            println!("Resolving value: {}", s);

            Value::String(s.clone())
        },

        Value::Identifier(name) => {
            if let Some(original_val) = module.variable_table.get(name) {
                return resolve_value(original_val, module, global_store);
            }

            if let Some(export_val) = find_export_value(name, global_store) {
                return resolve_value(&export_val, module, global_store);
            }

            println!("⚠️ Unresolved identifier '{}'", name);

            Value::Null
        }

        Value::Map(map) => {
            if let Some(Value::Identifier(entity)) = map.get("entity") {
                // SECTION Synth
                if entity == "synth" {
                    if let Some(Value::Map(synth_data)) = map.get("value") {
                        let resolved_waveform = synth_data
                            .get("waveform")
                            .map(|wf| resolve_value(wf, module, global_store))
                            .unwrap_or(Value::Null);

                        let resolved_params = synth_data
                            .get("parameters")
                            .map(|p| resolve_value(p, module, global_store))
                            .unwrap_or(Value::Map(HashMap::new()));

                        let mut result = HashMap::new();
                        result.insert("waveform".to_string(), resolved_waveform);
                        result.insert("parameters".to_string(), resolved_params);

                        return Value::Map(result);
                    }
                }
            }

            let mut resolved = HashMap::new();
            for (k, v) in map {
                resolved.insert(k.clone(), resolve_value(v, module, global_store));
            }

            Value::Map(resolved)
        }

        Value::Block(stmts) => {
            let resolved_stmts = stmts
                .iter()
                .map(|stmt| resolve_statement(stmt, module, &module.path, global_store))
                .collect();
            Value::Block(resolved_stmts)
        }

        other => other.clone(),
    }
}
