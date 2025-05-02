use crate::{frontend::lexer::Span, middle::types::Type};

use super::types::{Bind, Parameters};

pub trait LocalExtensions {
    fn is_undefined(&self) -> bool;
    fn is_mutable(&self) -> bool;
    fn get_span(&self) -> Span;
    fn get_type(&self) -> Type;
}

pub trait ConstantExtensions {
    fn get_type(&self) -> Type;
}

pub trait FunctionExtensions {
    fn ignore_more_args(&self) -> bool;
    fn get_type(&self) -> Type;
    fn get_parameters_size(&self) -> usize;
    fn get_parameters(&self) -> &Parameters;
}

pub trait BindingsExtensions {
    fn contains_binding(&self, name: &str) -> bool;
    fn get_bind(&self, name: &str) -> Bind;
}

pub trait BindExtensions {
    fn get_name(&self) -> &str;
    fn get_parameters_types(&self) -> &[Type];
    fn get_type(&self) -> Type;
}
