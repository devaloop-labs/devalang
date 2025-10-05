#![cfg(feature = "cli")]

use anyhow::Result;
use std::path::{Path, PathBuf};

pub struct ProjectPaths;

impl ProjectPaths {
    pub fn resolve(entry: Option<&Path>) -> Result<PathBuf> {
        let path = entry
            .map(PathBuf::from)
            .unwrap_or_else(|| std::env::current_dir().expect("current dir"));
        Ok(path)
    }
}
