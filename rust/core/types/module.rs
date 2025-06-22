use crate::core::types::{statement::Statement, store::{ExportTable, ImportTable, VariableTable}};

#[derive(Debug, Clone)]
pub struct Module {
    pub path: String,
    pub statements: Vec<Statement>,
    pub variable_table: VariableTable,
    pub export_table: ExportTable,
    pub import_table: ImportTable,
}