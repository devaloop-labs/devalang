/// Structured error information with details and suggestions
#[derive(Debug, Clone)]
pub struct StructuredError {
    /// Main error message
    pub message: String,
    /// File path where the error occurred
    pub file_path: Option<String>,
    /// Line number in the file
    pub line: Option<usize>,
    /// Column number in the file
    pub column: Option<usize>,
    /// Error type/category (e.g., "SyntaxError", "UnknownStatement", "RuntimeError")
    pub error_type: Option<String>,
    /// Optional "Did you mean ... ?" suggestion
    pub suggestion: Option<String>,
    /// Optional stack trace information
    pub stacktrace: Vec<String>,
}

impl StructuredError {
    /// Create a new structured error
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            file_path: None,
            line: None,
            column: None,
            error_type: None,
            suggestion: None,
            stacktrace: Vec::new(),
        }
    }

    /// Set file path
    pub fn with_file(mut self, path: impl Into<String>) -> Self {
        self.file_path = Some(path.into());
        self
    }

    /// Set line and column
    pub fn with_location(mut self, line: usize, column: usize) -> Self {
        self.line = Some(line);
        self.column = Some(column);
        self
    }

    /// Set error type
    pub fn with_type(mut self, error_type: impl Into<String>) -> Self {
        self.error_type = Some(error_type.into());
        self
    }

    /// Set suggestion
    pub fn with_suggestion(mut self, suggestion: impl Into<String>) -> Self {
        self.suggestion = Some(suggestion.into());
        self
    }

    /// Add a stack trace entry
    pub fn add_stacktrace(mut self, entry: impl Into<String>) -> Self {
        self.stacktrace.push(entry.into());
        self
    }

    /// Build the formatted error details for display (old format, kept for compatibility)
    pub fn build_details(&self) -> Vec<String> {
        let mut details = Vec::new();

        // Location: file:line:column with "path" label
        if let (Some(file), Some(line), Some(col)) = (&self.file_path, self.line, self.column) {
            details.push(format!("path: {file}:{line}:{col}"));
        } else if let (Some(file), Some(line)) = (&self.file_path, self.line) {
            details.push(format!("path: {file}:{line}"));
        } else if let Some(file) = &self.file_path {
            details.push(format!("path: {file}"));
        }

        // Error type with "code" label
        if let Some(ref error_type) = self.error_type {
            details.push(format!("code: {error_type}"));
        }

        // Stack trace entries
        for entry in &self.stacktrace {
            details.push(entry.clone());
        }

        // Suggestion with "help" label
        if let Some(ref suggestion) = self.suggestion {
            details.push(format!("help: {suggestion}"));
        }

        details
    }

    /// Build the formatted error details for colored display
    /// Returns tuples of (label, content) for the logger to format with colors
    pub fn build_colored_details(&self) -> Vec<(String, String)> {
        let mut details = Vec::new();

        // Location: file:line:column with "path" label
        if let (Some(file), Some(line), Some(col)) = (&self.file_path, self.line, self.column) {
            details.push(("path".to_string(), format!("{file}:{line}:{col}")));
        } else if let (Some(file), Some(line)) = (&self.file_path, self.line) {
            details.push(("path".to_string(), format!("{file}:{line}")));
        } else if let Some(file) = &self.file_path {
            details.push(("path".to_string(), file.clone()));
        }

        // Error type with "code" label
        if let Some(ref error_type) = self.error_type {
            details.push(("code".to_string(), error_type.clone()));
        }

        // Stack trace entries (without label)
        for entry in &self.stacktrace {
            details.push(("trace".to_string(), entry.clone()));
        }

        // Suggestion with "help" label
        if let Some(ref suggestion) = self.suggestion {
            details.push(("help".to_string(), suggestion.clone()));
        }

        details
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_structured_error_builder() {
        let error = StructuredError::new("Unknown statement")
            .with_file("test.deva")
            .with_location(5, 3)
            .with_type("SyntaxError")
            .with_suggestion("Did you mean 'sleep' ?");

        assert_eq!(error.message, "Unknown statement");
        assert_eq!(error.file_path, Some("test.deva".to_string()));
        assert_eq!(error.line, Some(5));
        assert_eq!(error.column, Some(3));
        assert_eq!(error.error_type, Some("SyntaxError".to_string()));
        assert_eq!(error.suggestion, Some("Did you mean 'sleep' ?".to_string()));
    }

    #[test]
    fn test_build_details() {
        let error = StructuredError::new("Test error")
            .with_file("test.deva")
            .with_location(10, 5)
            .with_type("RuntimeError")
            .add_stacktrace("at collector.rs:123")
            .with_suggestion("Did you mean 'print' ?");

        let details = error.build_details();
        assert!(details.iter().any(|d| d.contains("at:")));
        assert!(details.iter().any(|d| d.contains("code:")));
        assert!(details.iter().any(|d| d.contains("help:")));
    }

    #[test]
    fn test_build_colored_details() {
        let error = StructuredError::new("Test error")
            .with_file("test.deva")
            .with_location(10, 5)
            .with_type("RuntimeError")
            .with_suggestion("Did you mean 'print' ?");

        let details = error.build_colored_details();
        assert_eq!(details.len(), 3); // at, code, help
        assert_eq!(details[0].0, "at");
        assert_eq!(details[1].0, "code");
        assert_eq!(details[2].0, "help");
    }
}
