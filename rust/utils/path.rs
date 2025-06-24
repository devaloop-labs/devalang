use std::path::{ Component, Path };

pub fn find_entry_file(path: &str) -> Option<String> {
    let path = Path::new(path);

    // Check if the path is a file
    if path.is_file() {
        return Some(path.to_string_lossy().to_string());
    }

    // Check if the path is a directory
    if path.is_dir() {
        // Look for an index.deva file in the directory
        let index_path = path.join("index.deva");
        if index_path.is_file() {
            return Some(index_path.to_string_lossy().to_string());
        }
    }

    None
}

pub fn normalize_path(path: &str) -> String {
    let mut components = Vec::new();

    // Iterate through the components of the path
    for comp in Path::new(path).components() {
        match comp {
            Component::CurDir => {
                continue;
            }
            Component::Normal(c) => components.push(c),
            Component::RootDir => components.clear(),
            _ => {}
        }
    }

    // Join the components into a normalized path
    let normalized = components
        .iter()
        .map(|c| c.to_string_lossy())
        .collect::<Vec<_>>()
        .join("/");

    format!("./{}", normalized)
}
