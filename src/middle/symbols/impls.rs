use std::fmt::Display;

use crate::{frontend::lexer::Span, middle::types::Type};

use super::{
    traits::{
        BindExtensions, BindingsExtensions, ConstantExtensions, FunctionExtensions, LocalExtensions,
    },
    types::{Bind, Bindings, Constant, Function, Local, Parameters},
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

impl BindingsExtensions for Bindings<'_> {
    fn contains_binding(&self, name: &str) -> bool {
        self.iter().any(|binding| binding.0 == name)
    }

    fn get_bind(&self, name: &str) -> Bind {
        self.iter().find(|binding| binding.0 == name).unwrap()
    }
}

impl BindExtensions for Bind<'_> {
    fn get_name(&self) -> &str {
        self.0
    }

    fn get_parameters_types(&self) -> &[Type] {
        &self.2
    }

    fn get_type(&self) -> Type {
        self.1.clone()
    }
}
