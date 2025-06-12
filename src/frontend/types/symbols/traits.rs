use crate::frontend::{lexer::span::Span, types::lexer::ThrushType};

pub trait LocalSymbolExtensions {
    fn is_undefined(&self) -> bool;
    fn is_mutable(&self) -> bool;
    fn get_span(&self) -> Span;
    fn get_type(&self) -> ThrushType;
}

pub trait ConstantSymbolExtensions {
    fn get_type(&self) -> ThrushType;
}

pub trait LLISymbolExtensions {
    fn get_type(&self) -> ThrushType;
    fn get_span(&self) -> Span;
}

pub trait FunctionExtensions {
    fn get_type(&self) -> ThrushType;
}
