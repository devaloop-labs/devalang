use std::collections::HashMap;
use toml::Value as TomlValue;

use crate::core::{
    preprocessor::{module::Module, resolver::driver::resolve_statement},
    store::global::GlobalStore,
};

use devalang_types::Value;

fn find_export_value(name: &str, global_store: &GlobalStore) -> Option<Value> {
    for module in global_store.modules.values() {
        if let Some(val) = module.export_table.get_export(name) {
            return Some(val.clone());
        }
    }

    None
}

pub fn resolve_value(value: &Value, module: &Module, global_store: &mut GlobalStore) -> Value {
    match value {
        Value::String(s) => {
            // Keep raw strings as-is; they may be runtime-evaluated (e.g., expressions)
            Value::String(s.clone())
        }

        Value::Identifier(name) => {
            if let Some(original_val) = module.variable_table.get(name) {
                return resolve_value(original_val, module, global_store);
            }

            if let Some(export_val) = find_export_value(name, global_store) {
                return resolve_value(&export_val, module, global_store);
            }

            // Leave unresolved identifiers as-is; may be runtime-bound (e.g., foreach variable)
            Value::Identifier(name.clone())
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

                        // If waveform refers to a plugin synth (e.g., alias.synth),
                        // merge plugin-exported defaults (dynamic) into parameters and
                        // allow 'waveform' override from parameters map.
                        let mut final_waveform = resolved_waveform.clone();
                        let mut params_map = match resolved_params.clone() {
                            Value::Map(m) => m,
                            _ => HashMap::new(),
                        };

                        // Helper: convert TomlValue into runtime Value
                        fn toml_to_value(tv: &TomlValue) -> Value {
                            match tv {
                                TomlValue::String(s) => Value::String(s.clone()),
                                TomlValue::Integer(i) => Value::Number(*i as f32),
                                TomlValue::Float(f) => Value::Number(*f as f32),
                                TomlValue::Boolean(b) => Value::Boolean(*b),
                                TomlValue::Array(arr) => {
                                    Value::Array(arr.iter().map(toml_to_value).collect())
                                }
                                TomlValue::Table(t) => {
                                    let mut m = HashMap::new();
                                    for (k, v) in t.iter() {
                                        m.insert(k.clone(), toml_to_value(v));
                                    }
                                    Value::Map(m)
                                }
                                _ => Value::Null,
                            }
                        }

                        // Detect plugin alias from waveform string like "alias.synth" OR just "alias"
                        let (alias_opt, explicit_synth_export) = match &final_waveform {
                            Value::String(s) | Value::Identifier(s) => {
                                let parts: Vec<&str> = s.split('.').collect();
                                if parts.len() >= 2 && parts[1] == "synth" {
                                    (Some(parts[0].to_string()), true)
                                } else if parts.len() == 1 {
                                    (Some(parts[0].to_string()), false)
                                } else {
                                    (None, false)
                                }
                            }
                            _ => (None, false),
                        };

                        if let Some(alias) = alias_opt {
                            // Resolve alias -> plugin uri -> plugin info
                            if let Some(Value::String(uri)) = module.variable_table.get(&alias) {
                                if let Some(id) = uri.strip_prefix("devalang://plugin/") {
                                    let mut parts = id.split('.');
                                    let author = parts.next().unwrap_or("");
                                    let pname = parts.next().unwrap_or("");
                                    let key = format!("{}:{}", author, pname);
                                    if let Some((plugin_info, _wasm)) =
                                        global_store.plugins.get(&key)
                                    {
                                        // Merge defaults dynamically from exports
                                        for exp in &plugin_info.exports {
                                            // Skip entry named 'synth' which is used as the flag
                                            if exp.name == "synth" {
                                                continue;
                                            }
                                            if let Some(def) = &exp.default {
                                                let val = toml_to_value(def);
                                                // only apply if not overridden by user params
                                                params_map.entry(exp.name.clone()).or_insert(val);
                                            }
                                        }

                                        // If 'waveform' is provided in params (by user or default), use it
                                        if let Some(wf_val) = params_map.remove("waveform") {
                                            final_waveform =
                                                resolve_value(&wf_val, module, global_store);
                                        } else if let Some(wf_default) = plugin_info
                                            .exports
                                            .iter()
                                            .find(|e| e.name == "waveform")
                                            .and_then(|e| e.default.as_ref())
                                        {
                                            final_waveform = toml_to_value(wf_default);
                                        } else if explicit_synth_export {
                                            // keep as alias.synth if no default waveform
                                        } else {
                                            // If no explicit .synth in waveform, but alias is a plugin,
                                            // treat it as alias.synth by default to enable plugin synth usage
                                            final_waveform =
                                                Value::String(format!("{}.synth", alias));
                                        }
                                    }
                                }
                            }
                        }

                        let mut result = HashMap::new();
                        result.insert("waveform".to_string(), final_waveform);
                        result.insert("parameters".to_string(), Value::Map(params_map));

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
