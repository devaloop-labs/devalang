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
    // Prefer Cargo-provided manifest dir when available (build-time / cargo run)
    if let Ok(cargo_dir) = env::var("CARGO_MANIFEST_DIR") {
        let p = PathBuf::from(cargo_dir);
        if p.exists() {
            return Some(p);
        }
    }

    // At runtime (packaged binary) try to infer the package root from the
    // binary location: walk upward from the running executable and look for
    // common markers (package.json, project-version.json, Cargo.toml).
    if let Ok(exe_path) = env::current_exe() {
        if let Some(mut dir) = exe_path.parent().map(|p| p.to_path_buf()) {
            loop {
                if dir.join("package.json").exists()
                    || dir.join("project-version.json").exists()
                    || dir.join("Cargo.toml").exists()
                {
                    return Some(dir);
                }

                if let Some(parent) = dir.parent() {
                    dir = parent.to_path_buf();
                } else {
                    break;
                }
            }
        }
    }

    None
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

/// Finds the entry file given a path, returning the normalized path if found.
/// If the path is a directory, it looks for `index.deva` inside it.
/// Returns None if no valid entry file is found.
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

/// Normalizes a path to use forward slashes and removes redundant components.
pub fn normalize_path<P: AsRef<Path>>(path: P) -> String {
    let path_buf = PathBuf::from(path.as_ref());
    path_buf
        .components()
        .collect::<PathBuf>()
        .to_string_lossy()
        .replace('\\', "/")
}

/// Resolves a relative import path against a base path, normalizing the result.
pub fn resolve_relative_path(base: &str, import: &str) -> String {
    let base_path = Path::new(base).parent().unwrap_or_else(|| Path::new(""));
    let full_path = base_path.join(import);
    full_path
        .components()
        .collect::<PathBuf>()
        .to_string_lossy()
        .replace("\\", "/")
}
