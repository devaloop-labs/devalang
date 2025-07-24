use crate::core::{
    parser::statement::Statement,
    preprocessor::{ module::Module, resolver::value::resolve_value },
    store::global::GlobalStore,
};

pub fn resolve_let(
    stmt: &Statement,
    name: &str,
    module: &Module,
    path: &str,
    global_store: &mut GlobalStore
) -> Statement {
    let resolved_value = resolve_value(&stmt.value, module, global_store);

    global_store.variables.set(name.to_string(), resolved_value.clone());

    if let Some(current_mod) = global_store.modules.get_mut(path) {
        current_mod.variable_table.set(name.to_string(), resolved_value.clone());
    } else {
        eprintln!("[resolve_statement] ❌ Module path not found: {path}");
    }

    Statement {
        value: resolved_value,
        ..stmt.clone()
    }
}
