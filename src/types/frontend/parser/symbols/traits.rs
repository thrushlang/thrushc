use crate::{frontend::lexer::span::Span, types::frontend::lexer::types::ThrushType};

use super::types::MethodDef;

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
}

pub trait FunctionExtensions {
    fn get_type(&self) -> ThrushType;
}

pub trait MethodsExtensions {
    fn contains_method(&self, name: &str) -> bool;
    fn get_method(&self, name: &str) -> MethodDef;
}

pub trait MethodExtensions {
    fn get_name(&self) -> &str;
    fn get_parameters_types(&self) -> &[ThrushType];
    fn get_type(&self) -> ThrushType;
}
