use thrushc_span::Span;

use thrushc_token_type::TokenType;

pub trait TokenExtensions {
    fn get_lexeme(&self) -> &str;
    fn get_span(&self) -> Span;
    fn get_type(&self) -> TokenType;
    fn get_ascii_lexeme(&self) -> &str;
    fn get_lexeme_first_byte(&self) -> u64;
}
