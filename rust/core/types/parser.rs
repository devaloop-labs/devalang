use crate::core::types::{
    store::{ ExportTable, ImportTable, VariableTable },
    token::{ Token, TokenKind },
};

#[derive(Debug, Clone)]
pub struct Parser {
    pub tokens: Vec<Token>,
    pub token_index: usize,
    pub variable_table: VariableTable,
    pub export_table: ExportTable,
    pub import_table: ImportTable,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self {
            tokens,
            token_index: 0,
            variable_table: VariableTable::new(),
            export_table: ExportTable::new(),
            import_table: ImportTable::new(),
        }
    }

    pub fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.token_index)
    }

    pub fn next(&mut self) -> Option<&Token> {
        let tok = self.tokens.get(self.token_index);
        self.token_index += 1;
        tok
    }

    pub fn expect(&mut self, kind: TokenKind) -> Result<&Token, String> {
        let tok = self.next().ok_or("Unexpected end of input")?;
        if tok.kind == kind {
            Ok(tok)
        } else {
            Err(format!("Expected {:?}, got {:?}", kind, tok.kind))
        }
    }

    pub fn collect_until<F>(&mut self, condition: F) -> Vec<Token> where F: Fn(&Token) -> bool {
        let mut collected = Vec::new();
        while let Some(token) = self.peek() {
            if condition(token) {
                break;
            }
            collected.push(self.next().unwrap().clone());
        }
        collected
    }

    pub fn is_eof(&self) -> bool {
        self.token_index >= self.tokens.len()
    }
}
