pub mod module;
pub mod dependencies;

use std::{
    collections::{HashSet, VecDeque},
    fs,
};

use crate::core::{
    preprocessor::module::load_module_into_global_store,
    types::{store::GlobalStore, token::Token},
};

/// 🔁 Fonction principale de prétraitement :
/// Charge tous les fichiers à partir d’un fichier d’entrée, en suivant les @import
pub fn preprocess(entry_file: &str) -> GlobalStore {
    println!("📦 Collecting dependencies for: {}", entry_file);

    let files = collect_dependencies_recursively(entry_file);
    let mut store = GlobalStore::default();

    for file in &files {
        if let Err(e) = load_module_into_global_store(file, &mut store) {
            eprintln!("❌ Error while loading {}: {}", file, e);
        }
    }

    store
}

/// 🔍 Résout récursivement les fichiers @import depuis un fichier d’entrée
pub fn collect_dependencies_recursively(entry_file: &str) -> Vec<String> {
    let mut queue = VecDeque::new();
    let mut loaded = HashSet::new();

    queue.push_back(entry_file.to_string());

    while let Some(file_ref) = queue.pop_front() {
        if loaded.contains(&file_ref) {
            continue;
        }

        println!("📄 Processing file: {}", file_ref);

        let deps = get_direct_dependencies(&file_ref); // ✅ fix ici
        for dep in deps {
            queue.push_back(dep.clone());
        }

        loaded.insert(file_ref);
    }

    loaded.into_iter().collect()
}

/// 🔎 Analyse un fichier pour en extraire les chemins depuis les lignes `@import { ... } from "..."`
/// Ne parse pas, lit juste les lignes en brut
fn get_direct_dependencies(file: &str) -> Vec<String> {
    let content = match fs::read_to_string(file) {
        Ok(c) => c,
        Err(_) => {
            eprintln!("⚠️ Unable to read file: {}", file);
            return vec![];
        }
    };

    let mut deps = Vec::new();

    for line in content.lines() {
        let line = line.trim();

        if line.starts_with("@import") {
            if let Some(from_index) = line.find("from") {
                let after_from = line[from_index + 4..].trim();
                if after_from.starts_with('"') || after_from.starts_with('\'') {
                    let delimiter = after_from.chars().next().unwrap();
                    if let Some(end_quote) = after_from[1..].find(delimiter) {
                        let path = &after_from[1..=end_quote];
                        deps.push(path.to_string());
                    }
                }
            }
        }
    }

    println!("🔗 Direct dependencies for {}: {:?}", file, deps);
    deps
}
