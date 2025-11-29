use crate::core::diagnostic::span::Span;
use crate::front_end::lexer::tokentype::TokenType;
use crate::front_end::types::parser::stmts::traits::TokenExtensions;

#[derive(Debug)]
pub struct Token {
    pub lexeme: String,
    pub ascii: String,
    pub bytes: Vec<u8>,
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
        &self.ascii
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
    fn get_bytes_lexeme(&self) -> &[u8] {
        &self.bytes
    }

    #[inline]
    fn get_lexeme_first_byte(&self) -> u64 {
        self.lexeme.as_bytes()[0] as u64
    }
}
