use crate::core::types::{
    module::Module,
    parser::Parser,
    statement::{ Statement, StatementKind },
    store::{ ExportTable, GlobalStore, ImportTable },
    variable::VariableValue,
};

pub fn resolve_exports(statements: &[Statement], parser: &Parser) -> ExportTable {
    let mut export_table = parser.export_table.clone();

    for stmt in statements {
        if let StatementKind::Export = &stmt.kind {
            if let VariableValue::Array(tokens) = &stmt.value {
                for token in tokens {
                    let var_name = &token.lexeme;
                    if let Some(value) = parser.variable_table.variables.get(var_name) {
                        export_table.add_export(var_name.clone(), value.clone());
                    } else {
                        eprintln!("⚠️ Variable '{}' not found in scope, export skipped", var_name);
                    }
                }
            } else {
                eprintln!("⚠️ Unexpected value type in export: {:?}", stmt.value);
            }
        }
    }

    export_table
}

pub fn resolve_imports(module: &mut Module, global_store: &GlobalStore) -> ImportTable {
    let mut import_table = ImportTable::default();

    for stmt in &module.statements {
        if let StatementKind::Import { names, source } = &stmt.kind {
            if let Some(from_module) = global_store.modules.get(source) {
                for name in names {
                    if let Some(value) = from_module.export_table.exports.get(name) {
                        module.variable_table.variables.insert(name.clone(), value.clone());
                        import_table.add_import(name.clone(), value.clone());
                    } else {
                        eprintln!("⚠️ '{}' not found in exports of '{}'", name, source);
                    }
                }
            } else {
                eprintln!("⚠️ Module '{}' not found", source);
            }
        }
    }

    import_table
}
