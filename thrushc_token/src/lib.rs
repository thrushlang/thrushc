mod impls;
pub mod tokentype;
pub mod traits;

use thrushc_span::Span;

use crate::{tokentype::TokenType, traits::TokenExtensions};

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
        *self.lexeme.as_bytes().first().unwrap_or(&b'\0') as u64
    }
}
