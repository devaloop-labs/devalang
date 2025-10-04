use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct SourceFile {
    pub path: PathBuf,
    pub content: String,
}

impl SourceFile {
    pub fn from_str(content: impl Into<String>) -> Self {
        Self {
            path: PathBuf::new(),
            content: content.into(),
        }
    }

    pub fn from_path(path: impl AsRef<Path>) -> anyhow::Result<Self> {
        let path = path.as_ref();
        let content = std::fs::read_to_string(path)?;
        Ok(Self {
            path: path.to_path_buf(),
            content,
        })
    }

    pub fn iter_lines(&self) -> impl Iterator<Item = &str> {
        self.content.lines()
    }
}
