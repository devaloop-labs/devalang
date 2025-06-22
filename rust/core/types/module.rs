use crate::core::types::{
    statement::Statement,
    store::{ ExportTable, ImportTable, VariableTable },
    token::Token,
    variable::VariableValue,
};

#[derive(Debug, Clone)]
pub struct Module {
    pub path: String,
    pub tokens: Vec<Token>,
    pub statements: Vec<Statement>,
    pub variable_table: VariableTable,
    pub export_table: ExportTable,
    pub import_table: ImportTable,
}

impl Module {
    pub fn new(path: String) -> Self {
        Module {
            path,
            tokens: Vec::new(),
            statements: Vec::new(),
            variable_table: VariableTable::new(),
            export_table: ExportTable::new(),
            import_table: ImportTable::new(),
        }
    }

    pub fn add_statement(&mut self, statement: Statement) {
        self.statements.push(statement);
    }

    pub fn set_variable(&mut self, name: String, value: VariableValue) {
        self.variable_table.set(name, value);
    }

    pub fn get_variable(&self, name: &str) -> Option<&VariableValue> {
        self.variable_table.get(name)
    }
}
