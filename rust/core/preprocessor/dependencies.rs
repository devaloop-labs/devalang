use std::{ collections::{ HashSet, VecDeque }, fs };

/// 🔍 Résout récursivement les fichiers @import depuis un fichier d’entrée
pub fn collect_dependencies_recursively(entry_file: &str) -> Vec<String> {
    let mut queue = VecDeque::new();
    let mut loaded = HashSet::new();

    queue.push_back(entry_file.to_string());

    while let Some(file_ref) = queue.pop_front() {
        if loaded.contains(&file_ref) {
            continue;
        }

        let deps = get_direct_dependencies(&file_ref);
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

    deps
}
