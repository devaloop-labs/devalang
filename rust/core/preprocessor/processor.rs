use crate::core::{
    parser::{ statement::StatementKind, Parser },
    preprocessor::loader::ModuleLoader,
    shared::value::Value,
    store::global::GlobalStore,
};

pub fn process_modules(module_loader: &ModuleLoader, global_store: &mut GlobalStore) {
    for module in global_store.modules.values_mut() {
        for stmt in &module.statements {
            match &stmt.kind {
                StatementKind::Let { name } => {
                    module.variable_table.variables.insert(name.clone(), stmt.value.clone());
                }

                StatementKind::Load { source, alias } => {
                    module.variable_table.variables.insert(
                        alias.clone(),
                        Value::Sample(source.clone())
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
                    for name in names {
                        module.import_table.add_import(name.clone(), Value::String(source.clone()));
                    }
                }
                
                _ => {}
            }
        }
    }
}
