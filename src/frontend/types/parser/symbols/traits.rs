use crate::frontend::{lexer::span::Span, typesystem::types::Type};

pub trait LocalSymbolExtensions {
    fn is_undefined(&self) -> bool;
    fn is_mutable(&self) -> bool;
    fn get_span(&self) -> Span;
    fn get_type(&self) -> Type;
}

pub trait ConstantSymbolExtensions {
    fn get_type(&self) -> Type;
}

pub trait LLISymbolExtensions {
    fn get_type(&self) -> Type;
    fn get_span(&self) -> Span;
}

pub trait FunctionExtensions {
    fn get_type(&self) -> Type;
}
