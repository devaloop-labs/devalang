use crate::core::{
    lexer::token::Token,
    parser::statement::Statement,
    store::{ export::ExportTable, function::FunctionTable, import::ImportTable, variable::VariableTable },
};

#[derive(Debug, Clone)]
pub struct Module {
    pub path: String,
    pub resolved: bool,
    pub tokens: Vec<Token>,
    pub statements: Vec<Statement>,
    pub variable_table: VariableTable,
    pub functions: FunctionTable,
    pub export_table: ExportTable,
    pub import_table: ImportTable,
    pub content: String,
    pub current_dir: String,
}

impl Module {
    pub fn new(path: &str) -> Self {
        Module {
            path: path.to_string(),
            tokens: Vec::new(),
            statements: Vec::new(),
            variable_table: VariableTable::new(),
            functions: FunctionTable::new(),
            export_table: ExportTable::new(),
            import_table: ImportTable::new(),
            resolved: false,
            content: String::new(),
            current_dir: String::new(),
        }
    }

    pub fn is_resolved(&self) -> bool {
        self.resolved
    }

    pub fn set_resolved(&mut self, resolved: bool) {
        self.resolved = resolved;
    }

    pub fn add_token(&mut self, token: Token) {
        self.tokens.push(token);
    }

    pub fn add_statement(&mut self, statement: Statement) {
        self.statements.push(statement);
    }

    pub fn from_existing(path: &str, content: String) -> Self {
        let mut module = Module::new(path);
        module.content = content;
        module
    }
}
