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

        // module_name may be a relative path like "demos/hello-sound/index"
        // Split it into directory part and file part
        let module_path = Path::new(module_name);
        let (output_subdir, file_name) = if let Some(parent) = module_path.parent() {
            let parent_str = parent.to_string_lossy();
            if parent_str.is_empty() || parent_str == "." {
                (String::new(), module_name.to_string())
            } else {
                let file_name = module_path
                    .file_name()
                    .map(|s| s.to_string_lossy().to_string())
                    .unwrap_or_else(|| module_name.to_string());
                (parent_str.to_string(), file_name)
            }
        } else {
            (String::new(), module_name.to_string())
        };

        let full_ast_dir = if output_subdir.is_empty() {
            ast_dir.clone()
        } else {
            ast_dir.join(&output_subdir)
        };

        create_dir_all(&full_ast_dir).with_context(|| {
            format!(
                "failed to create AST output directory: {}",
                full_ast_dir.display()
            )
        })?;

        let file_path = full_ast_dir.join(format!("{}.json", file_name));
        let json = to_string_pretty(statements).context("failed to serialize AST")?;
        let mut file = File::create(&file_path)
            .with_context(|| format!("failed to create AST file: {}", file_path.display()))?;
        file.write_all(json.as_bytes())
            .with_context(|| format!("unable to write AST file: {}", file_path.display()))?;

        Ok(file_path)
    }
}
