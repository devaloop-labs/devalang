use crate::{
    core::{
        lexer::lex,
        parser::parse,
        preprocessor::collect_dependencies_recursively,
        types::{
            module::Module,
            parser::Parser,
            store::{ ExportTable, GlobalStore, ImportTable, VariableTable },
        },
    },
    resolve_exports,
    resolve_imports,
};

pub fn load_all_modules(entry_file: &str) -> GlobalStore {
    let mut global_store = GlobalStore::default();
    let files = collect_dependencies_recursively(entry_file);

    // Phase 1 – chargement + parsing
    for file in &files {
        if let Err(e) = load_module_into_global_store(file, &mut global_store) {
            eprintln!("❌ Error loading {}: {}", file, e);
        }
    }

    // Phase 2 – résolution des imports
    for file in &files {
        if let Some(module) = global_store.modules.clone().get_mut(file) {
            module.import_table = resolve_imports(module, &mut global_store);
        }
    }

    println!("✅ All modules loaded successfully!");

    global_store
}

// pub fn load_module_into_global_store(
//     path: &str,
//     global_store: &mut GlobalStore
// ) -> Result<(), String> {
//     if global_store.modules.contains_key(path) {
//         return Ok(()); // déjà chargé
//     }

//     // 1. Lire fichier
//     let content = std::fs::read_to_string(path).map_err(|_| format!("Cannot read file: {}", path))?;

//     // 2. Tokenize
//     let tokens = lex(content);

//     // NOTE Debug only
//     // println!("Lexed {:?}", tokens);

//     // 3. Parser
//     let mut parser = Parser::new(tokens.clone());
//     let statements = parse(tokens, &mut parser, global_store);

//     // 4. Résoudre les exports
//     let export_table = resolve_exports(&statements, &parser.variable_table);

//     // 5. Créer module sans imports (provisoirement)
//     let mut module = Module {
//         path: path.to_string(),
//         statements,
//         variable_table: VariableTable::default(),
//         export_table: ExportTable::new(),
//         import_table: ImportTable::default(), // rempli après
//     };

//     // 6. Résoudre les imports maintenant que tous les exports sont connus
//     module.variable_table = parser.variable_table.clone();
//     println!("Variable table : {:?}", parser.variable_table.clone());
//     module.export_table = export_table.clone();
//     println!("Export table : {:?}", export_table.clone());
//     module.import_table = resolve_imports(&module, global_store);

//     // NOTE Debug only
//     println!("Import table for {}: {:?}", path, module.import_table.imports);
//     println!("EXPORTS: {:?}", module.export_table.exports);

//     // 7. Injecter dans le GlobalStore
//     global_store.modules.insert(path.to_string(), module.clone());

//     println!("✅ Module loaded: {:?}", global_store.modules.get(path));

//     // NOTE Debug only
//     // for stmt in &module.statements {
//     //     println!("File: {}", path);
//     //     println!("{:?}", stmt);
//     // }

//     Ok(())
// }

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
    let statements = parse(tokens, &mut parser, global_store);

    let export_table = resolve_exports(&statements, &parser.variable_table);

    let module = Module {
        path: path.to_string(),
        statements,
        variable_table: parser.variable_table,
        export_table,
        import_table: ImportTable::default(),
    };

    // Résoudre les imports maintenant que tous les exports sont connus
    let mut module = module.clone();
    module.import_table = resolve_imports(&module, global_store);

    global_store.modules.insert(path.to_string(), module);

    Ok(())
}
