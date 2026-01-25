use thrushc_span::Span;

use crate::tokentype::TokenType;

pub trait TokenExtensions {
    fn get_lexeme(&self) -> &str;
    fn get_span(&self) -> Span;
    fn get_type(&self) -> TokenType;
    fn get_ascii_lexeme(&self) -> &str;
    fn get_lexeme_first_byte(&self) -> u64;
}

pub trait TokenTypeBuiltinExtensions {
    fn is_builtin(&self) -> bool;
}

pub trait TokenTypeExtensions {
    fn is_logical_operator(&self) -> bool;
    fn is_logical_gate(&self) -> bool;
    fn is_minus_minus_operator(&self) -> bool;
    fn is_plus_plus_operator(&self) -> bool;
    fn is_address(&self) -> bool;
    fn is_void(&self) -> bool;
    fn is_bool(&self) -> bool;
    fn is_array(&self) -> bool;
    fn is_ptr(&self) -> bool;
    fn is_float(&self) -> bool;
    fn is_const(&self) -> bool;
    fn is_fn_ref(&self) -> bool;
    fn is_integer(&self) -> bool;
    fn is_type(&self) -> bool;
    fn is_identifier(&self) -> bool;
}

pub trait TokenTypeAttributesExtensions {
    fn is_attribute(&self) -> bool;
}
