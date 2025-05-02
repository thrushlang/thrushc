use std::fmt::Display;

use crate::{frontend::lexer::Span, middle::types::Type};

use super::{
    traits::{ConstantExtensions, FunctionExtensions, LocalExtensions},
    types::{Constant, Function, Local, Parameters},
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

    fn get_parameters_size(&self) -> usize {
        self.1.get_size()
    }

    fn get_parameters(&self) -> &Parameters {
        &self.1
    }
}

impl Parameters {
    pub fn new(inner: Vec<Type>) -> Self {
        Self(inner)
    }

    pub fn get_size(&self) -> usize {
        self.0.len()
    }
}

impl Display for Parameters {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (index, kind) in self.0.iter().enumerate() {
            if index > 0 {
                write!(f, ", ")?;
            }

            write!(f, "{}", kind)?;
        }

        Ok(())
    }
}
