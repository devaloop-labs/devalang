#![cfg(feature = "cli")]

use anyhow::Result;
use std::path::{Path, PathBuf};

pub struct ProjectPaths;

impl ProjectPaths {
    pub fn resolve(entry: Option<&Path>) -> Result<PathBuf> {
        let path = match entry {
            Some(entry) => PathBuf::from(entry),
            None => std::env::current_dir()?,
        };
        Ok(path)
    }
}
