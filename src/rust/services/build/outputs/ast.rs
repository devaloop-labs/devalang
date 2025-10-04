#![cfg(feature = "cli")]

use std::fs::{File, create_dir_all};
use std::io::Write;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use serde_json::to_string_pretty;

use crate::language::syntax::ast::Statement;

#[derive(Debug, Clone, Copy, Default)]
pub struct AstBuilder;

impl AstBuilder {
    pub fn new() -> Self {
        Self
    }

    pub fn write(
        &self,
        statements: &[Statement],
        output_root: impl AsRef<Path>,
        module_name: &str,
    ) -> Result<PathBuf> {
        let output_root = output_root.as_ref();
        let ast_dir = output_root.join("ast");
        create_dir_all(&ast_dir).with_context(|| {
            format!(
                "failed to create AST output directory: {}",
                ast_dir.display()
            )
        })?;

        let file_path = ast_dir.join(format!("{}.json", module_name));
        let json = to_string_pretty(statements).context("failed to serialize AST")?;
        let mut file = File::create(&file_path)
            .with_context(|| format!("failed to create AST file: {}", file_path.display()))?;
        file.write_all(json.as_bytes())
            .with_context(|| format!("unable to write AST file: {}", file_path.display()))?;

        Ok(file_path)
    }
}
