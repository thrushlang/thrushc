use crate::{frontend::lexer::Span, middle::types::frontend::lexer::types::ThrushType};

use super::types::{Bind, ParametersTypes};

pub trait LocalSymbolExtensions {
    fn is_undefined(&self) -> bool;
    fn is_mutable(&self) -> bool;
    fn get_span(&self) -> Span;
    fn get_type(&self) -> ThrushType;
}

pub trait ConstantSymbolExtensions {
    fn get_type(&self) -> ThrushType;
}

pub trait FunctionExtensions {
    fn has_ignore_more_args(&self) -> bool;
    fn get_type(&self) -> ThrushType;
    fn get_parameters_size(&self) -> usize;
    fn get_parameters_types(&self) -> &ParametersTypes;
}

pub trait BindingsExtensions {
    fn contains_binding(&self, name: &str) -> bool;
    fn get_bind(&self, name: &str) -> Bind;
}

pub trait BindExtensions {
    fn get_name(&self) -> &str;
    fn get_parameters_types(&self) -> &[ThrushType];
    fn get_type(&self) -> ThrushType;
}
