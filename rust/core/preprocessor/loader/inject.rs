use crate::core::parser::statement::Statement;
use crate::core::plugin::loader::load_plugin;
use crate::core::preprocessor::module::Module;
use crate::core::store::global::GlobalStore;
use devalang_types::Value;
use std::{collections::HashMap, path::Path};

pub fn inject_bank_triggers(
    module: &mut Module,
    bank_name: &str,
    alias_override: Option<String>,
) -> Result<(), String> {
    let default_alias = bank_name
        .split('.')
        .next_back()
        .unwrap_or(bank_name)
        .to_string();
    let alias_ref = alias_override.as_deref().unwrap_or(&default_alias);

    let root = match devalang_utils::path::get_deva_dir() {
        Ok(dir) => dir,
        Err(_) => Path::new("./.deva").to_path_buf(),
    };

    // Try both plural and singular folder names and both layouts (flat and nested)
    let mut parsed_bankfile_opt: Option<devalang_types::BankFile> = None;
    let sds = ["banks", "bank"];
    for sd in &sds {
        // candidate: .deva/<sd>/<bank_name>/bank.toml (flat dir name)
        let candidate1 = root.join(sd).join(bank_name).join("bank.toml");
        if candidate1.exists() {
            let content = std::fs::read_to_string(&candidate1)
                .map_err(|e| format!("Failed to read '{}': {}", candidate1.display(), e))?;
            if let Ok(parsed) = toml::from_str::<devalang_types::BankFile>(&content) {
                parsed_bankfile_opt = Some(parsed);
                break;
            }
        }

        // If bank_name uses dot notation, also try nested layout: .deva/<sd>/<publisher>/<name>/bank.toml
        if bank_name.contains('.') {
            let mut it = bank_name.splitn(2, '.');
            let pubr = it.next().unwrap_or("");
            let nm = it.next().unwrap_or("");
            let candidate2 = root.join(sd).join(pubr).join(nm).join("bank.toml");
            if candidate2.exists() {
                let content = std::fs::read_to_string(&candidate2)
                    .map_err(|e| format!("Failed to read '{}': {}", candidate2.display(), e))?;
                if let Ok(parsed) = toml::from_str::<devalang_types::BankFile>(&content) {
                    parsed_bankfile_opt = Some(parsed);
                    break;
                }
            }
        }
    }

    let parsed_bankfile = match parsed_bankfile_opt {
        Some(p) => p,
        None => return Ok(()),
    };

    let mut bank_map = HashMap::new();

    for bank_trigger in parsed_bankfile.triggers.unwrap_or_default() {
        let entity_ref = bank_trigger
            .path
            .clone()
            .replace("\\", "/")
            .replace("./", "");
        let bank_trigger_path = format!("devalang://bank/{}/{}", bank_name, entity_ref);

        bank_map.insert(
            bank_trigger.name.clone(),
            Value::String(bank_trigger_path.clone()),
        );

        if module.variable_table.variables.contains_key(alias_ref) {
            eprintln!(
                "⚠️ Trigger '{}' already defined in module '{}', skipping injection.",
                alias_ref, module.path
            );
            continue;
        }

        module.variable_table.set(
            format!("{}.{}", alias_ref, bank_trigger.name),
            Value::String(bank_trigger_path.clone()),
        );
    }

    module
        .variable_table
        .set(alias_ref.to_string(), Value::Map(bank_map));

    Ok(())
}

pub fn extract_bank_decls(statements: &[Statement]) -> Vec<(String, Option<String>)> {
    let mut banks = Vec::new();

    for stmt in statements {
        if let crate::core::parser::statement::StatementKind::Bank { alias } = &stmt.kind {
            let name_opt = match &stmt.value {
                Value::String(s) => Some(s.clone()),
                Value::Identifier(s) => Some(s.clone()),
                Value::Number(n) => Some(n.to_string()),
                _ => None,
            };
            if let Some(name) = name_opt {
                banks.push((name, alias.clone()));
            }
        }
    }

    banks
}

pub fn extract_plugin_uses(statements: &[Statement]) -> Vec<(String, String)> {
    let mut plugins = Vec::new();

    for stmt in statements {
        if let crate::core::parser::statement::StatementKind::Use { name, alias } = &stmt.kind {
            let alias_name = alias
                .clone()
                .unwrap_or_else(|| name.split('.').next_back().unwrap_or(name).to_string());
            plugins.push((name.clone(), alias_name));
        }
    }

    plugins
}

pub fn load_plugin_and_register(
    module: &mut Module,
    plugin_name: &str,
    alias: &str,
    global_store: &mut GlobalStore,
) {
    let mut parts = plugin_name.split('.');
    let author = match parts.next() {
        Some(a) if !a.is_empty() => a,
        _ => {
            eprintln!("Invalid plugin name '{}': missing author", plugin_name);
            return;
        }
    };
    let name = match parts.next() {
        Some(n) if !n.is_empty() => n,
        _ => {
            eprintln!("Invalid plugin name '{}': missing name", plugin_name);
            return;
        }
    };
    if parts.next().is_some() {
        eprintln!(
            "Invalid plugin name '{}': expected <author>.<name>",
            plugin_name
        );
        return;
    }

    let expected_uri = format!("devalang://plugin/{}/{}", author, name);

    let root = match devalang_utils::path::get_deva_dir() {
        Ok(dir) => dir,
        Err(_) => Path::new("./.deva").to_path_buf(),
    };
    // Test both 'plugins' and 'plugin' folders
    let mut exists_locally = false;
    for sd in &["plugins", "plugin"] {
        let plugin_dir_preferred = root.join(sd).join(format!("{}/{}", author, name));
        let toml_path_preferred = plugin_dir_preferred.join("plugin.toml");
        let plugin_dir_fallback = root.join(sd).join(author).join(name);
        let toml_path_fallback = plugin_dir_fallback.join("plugin.toml");
        if toml_path_preferred.exists() || toml_path_fallback.exists() {
            exists_locally = true;
            break;
        }
    }

    if exists_locally {
        let cfg_opt = crate::config::ops::load_config(None);
        let mut declared = false;
        if let Some(cfg) = cfg_opt {
            if let Some(list) = cfg.plugins {
                declared = list.iter().any(|p| p.path == expected_uri);
            }
        }
        if !declared {
            module
                .statements
                .push(crate::core::parser::statement::Statement {
                    kind: crate::core::parser::statement::StatementKind::Error {
                        message: "plugin present in local files but missing in .devalang config"
                            .to_string(),
                    },
                    value: Value::Null,
                    indent: 0,
                    line: 0,
                    column: 0,
                });
            return;
        }
    }

    match load_plugin(author, name) {
        Ok((info, wasm)) => {
            // keep dotted form for config/expected URIs, but inject a slash form into variables
            let _uri_dot = format!("devalang://plugin/{}.{}", author, name);
            let uri_inject = format!("devalang://plugin/{}/{}", author, name);
            global_store
                .plugins
                .insert(format!("{}:{}", author, name), (info, wasm));
            module
                .variable_table
                .set(alias.to_string(), Value::String(uri_inject.clone()));
            global_store
                .variables
                .set(alias.to_string(), Value::String(uri_inject.clone()));

            if let Some((plugin_info, _)) =
                global_store.plugins.get(&format!("{}:{}", author, name))
            {
                for exp in &plugin_info.exports {
                    match exp.kind.as_str() {
                        "number" => {
                            if let Some(toml::Value::String(s)) = &exp.default {
                                if let Ok(n) = s.parse::<f32>() {
                                    module
                                        .variable_table
                                        .set(format!("{}.{}", alias, exp.name), Value::Number(n));
                                }
                            } else if let Some(toml::Value::Integer(i)) = &exp.default {
                                module.variable_table.set(
                                    format!("{}.{}", alias, exp.name),
                                    Value::Number(*i as f32),
                                );
                            } else if let Some(toml::Value::Float(f)) = &exp.default {
                                module.variable_table.set(
                                    format!("{}.{}", alias, exp.name),
                                    Value::Number(*f as f32),
                                );
                            }
                        }
                        "string" => {
                            if let Some(toml::Value::String(s)) = &exp.default {
                                module.variable_table.set(
                                    format!("{}.{}", alias, exp.name),
                                    Value::String(s.clone()),
                                );
                            }
                        }
                        "bool" => {
                            if let Some(toml::Value::Boolean(b)) = &exp.default {
                                module
                                    .variable_table
                                    .set(format!("{}.{}", alias, exp.name), Value::Boolean(*b));
                            }
                        }
                        "synth" => {
                            module.variable_table.set(
                                format!("{}.{}", alias, exp.name),
                                Value::String(format!("{}.{}", alias, exp.name)),
                            );
                        }
                        _ => {
                            if let Some(def) = &exp.default {
                                let val = match def {
                                    toml::Value::String(s) => Value::String(s.clone()),
                                    toml::Value::Integer(i) => Value::Number(*i as f32),
                                    toml::Value::Float(f) => Value::Number(*f as f32),
                                    toml::Value::Boolean(b) => Value::Boolean(*b),
                                    toml::Value::Array(arr) => Value::Array(
                                        arr.iter()
                                            .map(|v| match v {
                                                toml::Value::String(s) => Value::String(s.clone()),
                                                toml::Value::Integer(i) => Value::Number(*i as f32),
                                                toml::Value::Float(f) => Value::Number(*f as f32),
                                                toml::Value::Boolean(b) => Value::Boolean(*b),
                                                _ => Value::Null,
                                            })
                                            .collect(),
                                    ),
                                    toml::Value::Table(t) => {
                                        let mut m = std::collections::HashMap::new();
                                        for (k, v) in t.iter() {
                                            let vv = match v {
                                                toml::Value::String(s) => Value::String(s.clone()),
                                                toml::Value::Integer(i) => Value::Number(*i as f32),
                                                toml::Value::Float(f) => Value::Number(*f as f32),
                                                toml::Value::Boolean(b) => Value::Boolean(*b),
                                                _ => Value::Null,
                                            };
                                            m.insert(k.clone(), vv);
                                        }
                                        Value::Map(m)
                                    }
                                    _ => Value::Null,
                                };
                                if val != Value::Null {
                                    module
                                        .variable_table
                                        .set(format!("{}.{}", alias, exp.name), val);
                                }
                            }
                        }
                    }
                }
            }
        }
        Err(e) => eprintln!("Failed to load plugin {}: {}", plugin_name, e),
    }
}
