use std::{ collections::HashMap, path::Path };

use crate::core::{
    parser::{ driver::Parser, statement::StatementKind },
    preprocessor::{ loader::ModuleLoader, resolver::group },
    shared::value::Value,
    store::global::GlobalStore,
    utils::path::{normalize_path, resolve_relative_path},
};

pub fn process_modules(module_loader: &ModuleLoader, global_store: &mut GlobalStore) {
    for module in global_store.modules.values_mut() {
        for stmt in &module.statements {
            match &stmt.kind {
                StatementKind::Let { name } => {
                    module.variable_table.variables.insert(name.clone(), stmt.value.clone());
                }

                StatementKind::Load { source, alias } => {
                    let module_dir = Path::new(&module.path).parent().unwrap_or(Path::new(""));

                    let resolved_path = normalize_path(&module_dir.join(source));

                    module.variable_table.variables.insert(
                        alias.clone(),
                        Value::Sample(resolved_path)
                    );
                }

                StatementKind::Export { names, source } => {
                    for name in names {
                        if let Some(val) = module.variable_table.get(name) {
                            module.export_table.add_export(name.clone(), val.clone());
                        }
                    }
                }

                StatementKind::Import { names, source } => {
                    let resolved = resolve_relative_path(&module.path, source);
                    for name in names {
                        module.import_table.add_import(
                            name.clone(),
                            Value::String(resolved.clone())
                        );
                    }
                }

                StatementKind::Group => {
                    if let Value::Map(map) = &stmt.value {
                        if
                            let (Some(Value::String(name)), Some(Value::Block(body))) = (
                                map.get("identifier"),
                                map.get("body"),
                            )
                        {
                            let mut stored_map = HashMap::new();

                            stored_map.insert(
                                "identifier".to_string(),
                                Value::String(name.clone())
                            );

                            stored_map.insert("body".to_string(), Value::Block(body.clone()));

                            module.variable_table.set(name.to_string(), Value::Map(stored_map));
                        } else {
                            eprintln!("❌ Invalid group definition: {:?}", stmt.value);
                        }
                    }
                }

                _ => {}
            }
        }
    }
}
