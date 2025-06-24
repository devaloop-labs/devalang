pub mod module;
pub mod dependencies;
pub mod resolver;

use crate::core::{
    preprocessor::{
        dependencies::collect_dependencies_recursively,
        module::load_module_into_global_store,
    },
    types::{ store::GlobalStore },
};

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
