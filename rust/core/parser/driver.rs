use crate::core::{
    lexer::token::{ Token, TokenKind },
    parser::{
        handler::{
            arrow_call::parse_arrow_call,
            at::parse_at_token,
            bank::parse_bank_token,
            condition::parse_condition_token,
            dot::parse_dot_token,
            identifier::parse_identifier_token,
            loop_::parse_loop_token,
            tempo::parse_tempo_token,
        },
        statement::Statement,
    },
    shared::value::Value,
    store::global::GlobalStore,
};

#[derive(Debug, Clone, PartialEq)]
pub struct Parser {
    pub resolve_modules: bool,
    pub tokens: Vec<Token>,
    pub token_index: usize,
    pub current_module: String,
    pub previous: Option<Token>,
}

impl Parser {
    pub fn new() -> Self {
        Parser {
            resolve_modules: false,
            tokens: Vec::new(),
            token_index: 0,
            current_module: String::new(),
            previous: None,
        }
    }

    pub fn set_current_module(&mut self, module_path: String) {
        self.current_module = module_path;
    }

    pub fn advance(&mut self) -> Option<&Token> {
        if self.is_eof() {
            return None;
        }

        self.previous = self.tokens.get(self.token_index).cloned();
        self.token_index += 1;

        self.tokens.get(self.token_index - 1)
    }

    pub fn peek_is(&self, expected: &str) -> bool {
        self.peek().map_or(false, |t| t.lexeme == expected)
    }

    pub fn peek_nth(&self, n: usize) -> Option<&Token> {
        if self.token_index + n < self.tokens.len() {
            self.tokens.get(self.token_index + n)
        } else {
            None
        }
    }

    pub fn advance_if(&mut self, kind: TokenKind) -> bool {
        if self.match_token(kind) { true } else { false }
    }

    pub fn match_token(&mut self, kind: TokenKind) -> bool {
        if let Some(tok) = self.peek() {
            if tok.kind == kind {
                self.advance();
                return true;
            }
        }
        false
    }

    pub fn previous_clone(&self) -> Option<Token> {
        self.previous.clone()
    }

    pub fn parse_block(
        &self,
        tokens: Vec<Token>,
        global_store: &mut GlobalStore
    ) -> Vec<Statement> {
        let mut inner_parser = Parser {
            resolve_modules: self.resolve_modules,
            tokens,
            token_index: 0,
            current_module: self.current_module.clone(),
            previous: None,
        };

        inner_parser.parse_tokens(inner_parser.tokens.clone(), global_store)
    }

    pub fn parse_tokens(
        &mut self,
        tokens: Vec<Token>,
        global_store: &mut GlobalStore
    ) -> Vec<Statement> {
        self.tokens = tokens;
        self.token_index = 0;

        let mut statements = Vec::new();

        while !self.is_eof() {
            let token = match self.peek() {
                Some(t) => t.clone(),
                None => {
                    break;
                }
            };

            let statement = match &token.kind {
                TokenKind::At => parse_at_token(self, global_store),
                TokenKind::Identifier => {
                    if let Some(next) = self.peek_nth(1).cloned() {
                        if next.kind == TokenKind::Arrow {
                            parse_arrow_call(self, global_store)
                        } else {
                            parse_identifier_token(self, global_store)
                        }
                    } else {
                        parse_identifier_token(self, global_store)
                    }
                }
                TokenKind::Dot => parse_dot_token(self, global_store),
                TokenKind::Tempo => parse_tempo_token(self, global_store),
                TokenKind::Bank => parse_bank_token(self, global_store),
                TokenKind::Loop => parse_loop_token(self, global_store),
                TokenKind::If => parse_condition_token(self, global_store),

                | TokenKind::Else // Ignore else, already handled in `parse_condition_token`
                | TokenKind::Comment
                | TokenKind::Equals
                | TokenKind::Colon
                | TokenKind::Number
                | TokenKind::String
                | TokenKind::LBrace
                | TokenKind::RBrace
                | TokenKind::Comma
                | TokenKind::Newline
                | TokenKind::Dedent
                | TokenKind::Indent => {
                    self.advance();
                    continue;
                }

                TokenKind::EOF => {
                    break;
                }

                _ => {
                    println!("Unhandled token: {:?}", token);
                    self.advance();
                    Statement::unknown()
                }
            };

            statements.push(statement);
        }

        statements
    }

    pub fn check_token(&self, kind: TokenKind) -> bool {
        self.peek().map_or(false, |t| t.kind == kind)
    }

    pub fn parse_map_value(&mut self) -> Option<Value> {
        if !self.match_token(TokenKind::LBrace) {
            return None;
        }

        let mut map = std::collections::HashMap::new();

        while !self.check_token(TokenKind::RBrace) && !self.is_eof() {
            let key = if let Some(token) = self.advance() {
                token.lexeme.clone()
            } else {
                break;
            };

            if !self.match_token(TokenKind::Colon) {
                println!("Expected ':' after map key '{}'", key);
                break;
            }

            let value = if let Some(token) = self.peek_clone() {
                match token.kind {
                    TokenKind::String => {
                        self.advance();
                        Value::String(token.lexeme.clone())
                    }
                    TokenKind::Number => {
                        self.advance();
                        Value::Number(token.lexeme.parse().unwrap_or(0.0))
                    }
                    TokenKind::Identifier => {
                        self.advance();
                        Value::Identifier(token.lexeme.clone())
                    }
                    _ => {
                        println!("Unexpected token in map value: {:?}", token);
                        Value::Null
                    }
                }
            } else {
                Value::Null
            };

            map.insert(key, value);
        }

        if !self.match_token(TokenKind::RBrace) {
            println!("Expected '}}' at end of map");
        }

        Some(Value::Map(map))
    }

    pub fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.token_index)
    }

    pub fn peek_clone(&self) -> Option<Token> {
        self.tokens.get(self.token_index).cloned()
    }

    pub fn expect(&mut self, kind: TokenKind) -> Result<&Token, String> {
        let tok = self.advance().ok_or("Unexpected end of input")?;
        if tok.kind == kind {
            Ok(tok)
        } else {
            Err(format!("Expected {:?}, got {:?}", kind, tok.kind))
        }
    }

    pub fn collect_block_tokens(&mut self, base_indent: usize) -> Vec<Token> {
        let mut tokens = Vec::new();

        while let Some(tok) = self.peek() {
            if tok.indent <= base_indent && tok.kind != TokenKind::Newline {
                break;
            }
            tokens.push(self.advance().unwrap().clone());
        }

        tokens
    }

    pub fn collect_until<F>(&mut self, condition: F) -> Vec<Token> where F: Fn(&Token) -> bool {
        let mut collected = Vec::new();
        while let Some(token) = self.peek() {
            if token.kind == TokenKind::Newline || token.kind == TokenKind::Indent {
                self.advance(); // Skip newlines and indents
                continue;
            }
            if token.kind == TokenKind::EOF {
                break;
            }
            if condition(token) {
                break;
            }
            collected.push(self.advance().unwrap().clone());
        }
        collected
    }

    pub fn is_eof(&self) -> bool {
        self.token_index >= self.tokens.len()
    }

    pub fn parse_block_until_next_else(
        &mut self,
        base_indent: usize,
        global_store: &mut GlobalStore
    ) -> Vec<Statement> {
        let mut block_tokens = Vec::new();

        while let Some(tok) = self.peek() {
            // Stop if we encounter an 'else' at same indent level
            if tok.lexeme == "else" && tok.indent == base_indent {
                break;
            }
            block_tokens.push(self.advance().unwrap().clone());
        }

        self.parse_block(block_tokens, global_store)
    }

    pub fn parse_condition_until_colon(&mut self) -> Option<Value> {
        let tokens = self.collect_until(|t| t.kind == TokenKind::Colon);
        if tokens.is_empty() {
            return None;
        }

        let condition = tokens
            .iter()
            .map(|t| t.lexeme.clone())
            .collect::<Vec<_>>()
            .join(" ");

        Some(Value::String(condition))
    }

    pub fn parse_block_until_else_or_dedent(
        &mut self,
        base_indent: usize,
        global_store: &mut GlobalStore
    ) -> Vec<Statement> {
        let mut tokens = Vec::new();

        while let Some(tok) = self.peek() {
            if tok.lexeme == "else" && tok.indent == base_indent {
                break;
            }
            if tok.indent < base_indent && tok.kind != TokenKind::Newline {
                break;
            }
            tokens.push(self.advance().unwrap().clone());
        }

        self.parse_block(tokens, global_store)
    }
}
