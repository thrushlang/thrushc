use crate::{frontend::lexer::Span, middle::types::Type};

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
    fn is_mutable(&self) -> bool;
    fn ignore_more_args(&self) -> bool;
    fn get_type(&self) -> Type;
}
