use crate::core::{ preprocessor::module::Module, shared::value::Value, store::global::GlobalStore };

pub fn is_valid_entity(entity: &str, module: &Module, global_store: &GlobalStore) -> bool {
    let built_ins = ["kick", "snare", "hat", "clap"];

    if built_ins.contains(&entity) {
        return true;
    }

    if let Some(val) = module.variable_table.get(entity) {
        match val {
            Value::Sample(_) => true,
            _ => false,
        }
    } else {
        false
    }
}

pub fn is_valid_identifier(ident: &str, module: &Module) -> bool {
    let built_ins = ["auto"];

    if built_ins.contains(&ident) {
        return true;
    }

    if let Some(val) = module.variable_table.get(ident) {
        match val {
            Value::Identifier(_) => true,
            _ => false,
        }
    } else {
        false
    }
}
