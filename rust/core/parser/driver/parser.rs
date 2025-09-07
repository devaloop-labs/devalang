use crate::core::{
    lexer::token::{Token, TokenKind},
    parser::statement::Statement,
    store::global::GlobalStore,
};
use devalang_types::Value;

#[derive(Debug, Clone, PartialEq)]
pub struct Parser {
    pub resolve_modules: bool,
    pub tokens: Vec<Token>,
    pub token_index: usize,
    pub current_module: String,
    pub previous: Option<Token>,
}

impl Default for Parser {
    fn default() -> Self {
        Self::new()
    }
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
        crate::core::parser::driver::cursor::advance_impl(self)
    }

    pub fn peek_is(&self, expected: &str) -> bool {
        crate::core::parser::driver::cursor::peek_is_impl(self, expected)
    }

    pub fn peek_nth(&self, n: usize) -> Option<&Token> {
        crate::core::parser::driver::cursor::peek_nth_impl(self, n)
    }

    pub fn peek_nth_kind(&self, n: usize) -> Option<TokenKind> {
        crate::core::parser::driver::cursor::peek_nth_kind_impl(self, n)
    }

    pub fn advance_if(&mut self, kind: TokenKind) -> bool {
        crate::core::parser::driver::cursor::advance_if_impl(self, kind)
    }

    pub fn match_token(&mut self, kind: TokenKind) -> bool {
        crate::core::parser::driver::cursor::match_token_impl(self, kind)
    }

    pub fn previous_clone(&self) -> Option<Token> {
        crate::core::parser::driver::cursor::previous_clone_impl(self)
    }

    pub fn parse_block(
        &self,
        tokens: Vec<Token>,
        global_store: &mut GlobalStore,
    ) -> Vec<Statement> {
        crate::core::parser::driver::block::parse_block(self, tokens, global_store)
    }

    pub fn parse_tokens(
        &mut self,
        tokens: Vec<Token>,
        global_store: &mut GlobalStore,
    ) -> Vec<Statement> {
        crate::core::parser::driver::driver_impl::parse_tokens_impl(self, tokens, global_store)
    }

    pub fn check_token(&self, kind: TokenKind) -> bool {
        crate::core::parser::driver::cursor::peek_impl(self).is_some_and(|t| t.kind == kind)
    }

    pub fn peek_kind(&self) -> Option<TokenKind> {
        crate::core::parser::driver::cursor::peek_impl(self).map(|t| t.kind.clone())
    }

    pub fn parse_map_value(&mut self) -> Option<Value> {
        // Delegated to parse_map.rs
        crate::core::parser::driver::parse_map::parse_map_value(self)
    }

    // Parse an array value like [1, 2, 3] or ["a", b]
    pub fn parse_array_value(&mut self) -> Option<Value> {
        // delegated to parse_array.rs
        crate::core::parser::driver::parse_array::parse_array_value(self)
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
        crate::core::parser::driver::block::collect_block_tokens(self, base_indent)
    }

    pub fn collect_until<F>(&mut self, condition: F) -> Vec<Token>
    where
        F: Fn(&Token) -> bool,
    {
        crate::core::parser::driver::block::collect_until(self, condition)
    }

    pub fn is_eof(&self) -> bool {
        self.token_index >= self.tokens.len()
    }

    pub fn parse_block_until_next_else(
        &mut self,
        base_indent: usize,
        global_store: &mut GlobalStore,
    ) -> Vec<Statement> {
        crate::core::parser::driver::block::parse_block_until_next_else(
            self,
            base_indent,
            global_store,
        )
    }

    pub fn parse_condition_until_colon(&mut self) -> Option<Value> {
        crate::core::parser::driver::driver_impl::parse_condition_until_colon_impl(self)
    }

    pub fn parse_block_until_else_or_dedent(
        &mut self,
        base_indent: usize,
        global_store: &mut GlobalStore,
    ) -> Vec<Statement> {
        crate::core::parser::driver::block::parse_block_until_else_or_dedent(
            self,
            base_indent,
            global_store,
        )
    }
}
