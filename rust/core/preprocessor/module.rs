use crate::core::{
    lexer::lex,
    parser::{ parse_with_resolving, parse_without_resolving },
    preprocessor::{
        collect_dependencies_recursively,
        resolver::{ resolve_exports, resolve_imports },
    },
    types::{ module::Module, parser::Parser, store::{ ExportTable, GlobalStore, ImportTable } },
};

/// Charge tous les fichiers depuis le fichier d’entrée, en suivant les @import
/// Phase 1 : parse + export + stockage
/// Phase 2 : résolution des imports (quand tous les modules sont chargés)
pub fn load_all_modules(entry_file: &str) -> GlobalStore {
    let mut global_store = GlobalStore::default();
    let files = collect_dependencies_recursively(entry_file);

    // Phase 1 – parsing et export pour tous les fichiers
    for file in &files {
        if let Err(e) = load_module_into_global_store(file, &mut global_store) {
            eprintln!("❌ Error loading {}: {}", file, e);
        }
    }

    // Phase 2 – résolution des imports
    for file in &files {
        if let Some(module) = global_store.modules.clone().get_mut(file) {
            let imports = resolve_imports(module, &mut global_store);
            module.import_table = imports.clone();

            let global_store_module = global_store.modules.get(&file.clone().to_string());
            if let Some(global_store_module_found) = global_store_module {
                // On met à jour le module dans le store global
                global_store.insert_module(file.to_string(), module.clone());
            } else {
                eprintln!("❌ Module {} not found in global store after import resolution", file);
            }
        }
    }

    global_store
}

/// Parse un fichier, enregistre ses exports, et l’insère dans le GlobalStore
/// ⚠ Ne résout pas les imports ici !
pub fn load_module_into_global_store(
    path: &str,
    global_store: &mut GlobalStore
) -> Result<(), String> {
    if global_store.modules.contains_key(path) {
        return Ok(()); // déjà chargé
    }

    let content = std::fs::read_to_string(path).map_err(|_| format!("Cannot read file: {}", path))?;

    let tokens = lex(content);
    let mut parser = Parser::new(tokens.clone());

    // 🔄 Mettre à jour le contexte du module courant
    parser.current_module = path.to_string();

    // Parsing
    let raw_statements = parse_without_resolving(tokens.clone(), &mut parser, global_store);

    // Récupération des exports
    let export_table = resolve_exports(&raw_statements, &mut parser);

    // Construction du module sans les imports (ajoutés ensuite)
    let mut module = Module {
        path: path.to_string(),
        tokens: tokens.clone(),
        statements: raw_statements,
        variable_table: parser.variable_table.clone(),
        export_table: export_table.clone(),
        import_table: parser.import_table.clone(),
    };

    // On met à jour le module dans le store global
    global_store.insert_module(path.to_string(), module.clone());
    
    // let new_statements = parse_with_resolving(tokens, &mut parser, global_store);
    // // On met à jour les déclarations du module avec les nouvelles déclarations résolues
    // module.statements = new_statements;

    // global_store.update_module(path.to_string(), module);

    Ok(())
}
