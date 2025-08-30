use std::{
    env, fs,
    path::{Path, PathBuf},
};

pub const DEVALANG_CONFIG: &str = ".devalang";
pub const DEVA_DIR: &str = ".deva";

/// Returns the current working directory.
pub fn get_cwd() -> PathBuf {
    // In wasm (and some restricted environments) `env::current_dir()` is unsupported
    // and will return an error. Avoid panicking here and fall back to `.` so the
    // runtime can still operate in a virtual environment.
    env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
}

/// Returns true if the given directory looks like a devalang project root.
/// Preference is given to the presence of `.devalang` (config file),
/// but falling back to a `.deva` directory is allowed.
pub fn is_project_root(dir: &Path) -> bool {
    let config = dir.join(DEVALANG_CONFIG);
    if config.is_file() {
        return true;
    }
    let deva = dir.join(DEVA_DIR);
    deva.is_dir()
}

/// Walks upward from `start` to locate the first directory considered a project root.
pub fn find_project_root_from(start: &Path) -> Option<PathBuf> {
    for ancestor in start.ancestors() {
        if is_project_root(ancestor) {
            return Some(ancestor.to_path_buf());
        }
    }
    None
}

/// Finds the project root from the current working directory.
pub fn find_project_root() -> Option<PathBuf> {
    find_project_root_from(&get_cwd())
}

/// Finds the package root using the `CARGO_MANIFEST_DIR` env var set by Cargo.
pub fn get_package_root() -> Option<PathBuf> {
    let cargo_dir = env::var("CARGO_MANIFEST_DIR").ok()?;
    Some(PathBuf::from(cargo_dir))
}

/// Gets the project root or returns a descriptive error if not found.
pub fn get_project_root() -> Result<PathBuf, String> {
    find_project_root()
        .ok_or_else(|| "Project root not found. Run 'devalang init' in your project.".to_string())
}

/// Returns the path to `.devalang` in the project root, ensuring it exists.
pub fn get_devalang_config_path() -> Result<PathBuf, String> {
    let root = get_project_root()?;
    let config_path = root.join(DEVALANG_CONFIG);
    if !config_path.exists() {
        return Err(format!(
            "Config file not found at '{}'. Please run 'devalang init' before continuing.",
            config_path.display()
        ));
    }
    Ok(config_path)
}

/// Returns the `.deva` directory inside the project root (without creating it).
pub fn get_deva_dir() -> Result<PathBuf, String> {
    let root = get_project_root()?;
    Ok(root.join(DEVA_DIR))
}

/// Ensures the `.deva` directory exists in the project root and returns its path.
pub fn ensure_deva_dir() -> Result<PathBuf, String> {
    let deva = get_deva_dir()?;
    if !deva.exists() {
        fs::create_dir_all(&deva).map_err(|e| {
            format!(
                "Failed to create Deva directory '{}': {}",
                deva.display(),
                e
            )
        })?;
    }
    Ok(deva)
}
