mod impls;
pub mod traits;

use thrustc_span::Span;

#[cfg(feature = "fuzz")]
use arbitrary::Arbitrary;

use thrustc_token_type::TokenType;

use crate::traits::TokenExtensions;

#[cfg_attr(feature = "fuzz", derive(Arbitrary))]
#[derive(Debug)]
pub struct Token {
    pub lexeme: String,
    pub ascii: String,
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
    fn get_lexeme_first_byte(&self) -> u64 {
        *self.lexeme.as_bytes().first().unwrap_or(&b'\0') as u64
    }
}
