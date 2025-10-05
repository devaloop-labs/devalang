#![cfg(feature = "cli")]

use anyhow::Result;
use std::{
    env, fs,
    path::{Path, PathBuf},
};

pub const DEVALANG_CONFIG: &str = ".devalang";
pub const DEVA_DIR: &str = ".deva";

/// Returns the current working directory.
pub fn get_cwd() -> PathBuf {
    env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
}

/// Returns true if the given directory looks like a devalang project root.
/// Checks for the presence of `.devalang` config file or `.deva` directory.
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

/// Gets the project root or returns an error if not found.
pub fn get_project_root() -> Result<PathBuf> {
    find_project_root()
        .ok_or_else(|| anyhow::anyhow!("Project root not found. Run 'devalang init' in your project or create a .deva directory."))
}

/// Returns the `.deva` directory inside the project root (without creating it).
pub fn get_deva_dir() -> Result<PathBuf> {
    let root = get_project_root()?;
    Ok(root.join(DEVA_DIR))
}

/// Ensures the `.deva` directory exists in the project root and returns its path.
pub fn ensure_deva_dir() -> Result<PathBuf> {
    let deva = get_deva_dir()?;
    if !deva.exists() {
        fs::create_dir_all(&deva).map_err(|e| {
            anyhow::anyhow!(
                "Failed to create .deva directory '{}': {}",
                deva.display(),
                e
            )
        })?;
    }
    Ok(deva)
}

/// Gets the home directory's .deva (for user-level addons)
pub fn get_home_deva_dir() -> Result<PathBuf> {
    let home_dir =
        dirs::home_dir().ok_or_else(|| anyhow::anyhow!("Cannot determine home directory"))?;
    Ok(home_dir.join(".deva"))
}

/// Ensures the home directory's .deva exists
pub fn ensure_home_deva_dir() -> Result<PathBuf> {
    let deva = get_home_deva_dir()?;
    if !deva.exists() {
        fs::create_dir_all(&deva).map_err(|e| {
            anyhow::anyhow!(
                "Failed to create home .deva directory '{}': {}",
                deva.display(),
                e
            )
        })?;
    }
    Ok(deva)
}
