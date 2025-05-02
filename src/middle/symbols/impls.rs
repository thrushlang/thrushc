use crate::{frontend::lexer::Span, middle::types::Type};

use super::{
    traits::{ConstantExtensions, FunctionExtensions, LocalExtensions},
    types::{Constant, Function, Local},
};

impl LocalExtensions for Local<'_> {
    fn is_undefined(&self) -> bool {
        self.2
    }

    fn is_mutable(&self) -> bool {
        self.1
    }

    fn get_span(&self) -> Span {
        self.3
    }

    fn get_type(&self) -> Type {
        self.0.clone()
    }
}

impl ConstantExtensions for Constant<'_> {
    fn get_type(&self) -> Type {
        self.0.clone()
    }
}

impl FunctionExtensions for Function<'_> {
    fn ignore_more_args(&self) -> bool {
        self.2
    }

    fn get_type(&self) -> Type {
        self.0.clone()
    }
}
