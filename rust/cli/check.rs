use crate::{
    core::{ debugger::Debugger, preprocessor::module::load_all_modules },
    runner::executer::execute_statements,
};

pub fn handle_check_command(entry: String, output: String) -> () {
    let entry_file = "./examples/index.deva";
    
    println!("🔍 Checking entry file: {}", entry_file);

    // 📦 Charge tous les modules + résout les imports
    let global_store = load_all_modules(entry_file);

    // ✅ Affichage des modules et de leur contenu
    // println!("\n✅ Résumé des modules chargés :\n");
    // for (path, module) in &global_store.modules {
    //     println!("📁 {}", path);
    //     println!("  ▸ {} statements", module.statements.len());
    //     println!("  🔹 Variables: {:?}", module.variable_table.variables);
    //     println!("  🔸 Exports  : {:?}", module.export_table.exports);
    //     println!("  🔸 Imports  : {:?}", module.import_table.imports);
    //     println!();

    //     for stmt in &module.statements {
    //         println!("    → {:?}", stmt);
    //     }
    //     println!("\n-----------------------------\n");
    // }

    if let Some(module) = global_store.modules.get("./examples/index.deva") {
        let module_clone = module.clone();
        let debugger = Debugger::new(&module_clone);

        // Exécute les statements du module
        let resolved_statements = execute_statements(&module_clone, &debugger);

        // Exécute le débogueur
        debugger.write_files("./output/debug/", resolved_statements);
    }
}
