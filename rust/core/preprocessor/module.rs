use crate::core::{
    lexer::driver::lex,
    parser::{ parse_without_resolving },
    preprocessor::{
        collect_dependencies_recursively,
        resolver::{ resolve_exports, resolve_imports },
    },
    types::{ module::Module, parser::Parser, store::{ ExportTable, GlobalStore, ImportTable } },
};

pub fn load_all_modules(entry_file: &str) -> GlobalStore {
    let mut global_store = GlobalStore::default();
    let files = collect_dependencies_recursively(entry_file);

    for file in &files {
        if let Err(e) = load_module_into_global_store(file, &mut global_store) {
            eprintln!("❌ Error loading {}: {}", file, e);
        }
    }

    for file in &files {
        if let Some(module) = global_store.modules.clone().get_mut(file) {
            let imports = resolve_imports(module, &mut global_store);
            module.import_table = imports.clone();

            let global_store_module = global_store.modules.get(&file.clone().to_string());
            if let Some(global_store_module_found) = global_store_module {
                global_store.insert_module(file.to_string(), module.clone());
            } else {
                eprintln!("❌ Module {} not found in global store after import resolution", file);
            }
        }
    }

    global_store
}

pub fn load_module_into_global_store(
    path: &str,
    global_store: &mut GlobalStore
) -> Result<(), String> {
    if global_store.modules.contains_key(path) {
        return Ok(());
    }

    let content = std::fs::read_to_string(path).map_err(|_| format!("Cannot read file: {}", path))?;

    let tokens = lex(content);

    let mut parser = Parser::new(tokens.clone());

    parser.current_module = path.to_string();

    let raw_statements = parse_without_resolving(tokens.clone(), &mut parser, global_store);

    let export_table = resolve_exports(&raw_statements, &mut parser);

    let mut module = Module {
        path: path.to_string(),
        tokens: tokens.clone(),
        statements: raw_statements,
        variable_table: parser.variable_table.clone(),
        export_table: export_table.clone(),
        import_table: parser.import_table.clone(),
    };

    global_store.insert_module(path.to_string(), module.clone());

    Ok(())
}
