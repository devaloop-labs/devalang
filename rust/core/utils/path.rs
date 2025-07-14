use std::path::{ Component, Path, PathBuf };

pub fn find_entry_file(entry: &str) -> Option<String> {
    let path = Path::new(entry);

    if path.is_file() {
        return Some(normalize_path(entry));
    }

    if path.is_dir() {
        let candidate = path.join("index.deva");
        if candidate.exists() {
            return Some(normalize_path(&candidate));
        }
    }

    None
}

pub fn normalize_path<P: AsRef<Path>>(path: P) -> String {
    let path_buf = PathBuf::from(path.as_ref());
    path_buf.components().collect::<PathBuf>().to_string_lossy().replace('\\', "/")
}

pub fn resolve_relative_path(base: &str, import: &str) -> String {
    let base_path = Path::new(base)
        .parent()
        .unwrap_or_else(|| Path::new(""));
    let full_path = base_path.join(import);
    full_path.components().collect::<PathBuf>().to_string_lossy().replace("\\", "/")
}
