use crate::front_end::{
    lexer::tokentype::TokenType, types::parser::stmts::traits::TokenExtensions,
};

use super::span::Span;

#[derive(Debug)]
pub struct Token {
    pub lexeme: String,
    pub ascii_lexeme: String,
    pub kind: TokenType,
    pub span: Span,
}

impl TokenExtensions for Token {
    #[inline]
    fn get_lexeme(&self) -> &str {
        &self.lexeme
    }

    #[inline]
    fn get_ascii_lexeme(&self) -> &str {
        &self.ascii_lexeme
    }

    #[inline]
    fn get_span(&self) -> Span {
        self.span
    }

    #[inline]
    fn get_type(&self) -> TokenType {
        self.kind
    }

    #[inline]
    fn get_lexeme_first_byte(&self) -> u64 {
        self.lexeme.as_bytes()[0] as u64
    }
}
