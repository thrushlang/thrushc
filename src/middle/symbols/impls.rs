use std::fmt::Display;

use crate::{
    frontend::lexer::Span,
    middle::{
        statement::{
            StructFields,
            traits::{FoundSymbolEither, FoundSymbolExtension, StructExtensions},
        },
        types::Type,
    },
    standard::error::ThrushCompilerError,
};

use super::{
    traits::{
        BindExtensions, BindingsExtensions, ConstantExtensions, FunctionExtensions, LocalExtensions,
    },
    types::{Bind, Bindings, Constant, FoundSymbolId, Function, Local, Parameters, Struct},
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

impl<'a> StructExtensions<'a> for Struct<'a> {
    fn contains_field(&self, name: &str) -> bool {
        self.1.iter().any(|field| field.0 == name)
    }

    fn get_field_type(&self, name: &str) -> Option<Type> {
        if let Some(field) = self.1.iter().find(|field| field.0 == name) {
            let field_type: Type = field.1.clone();
            return Some(field_type);
        }

        None
    }

    fn get_fields(&self) -> StructFields<'a> {
        (self.0, self.1.clone())
    }

    fn get_bindings(&self) -> Bindings<'a> {
        self.3.clone()
    }
}

impl FoundSymbolExtension for FoundSymbolId<'_> {
    fn is_structure(&self) -> bool {
        self.0.is_some()
    }

    fn is_function(&self) -> bool {
        self.1.is_some()
    }

    fn is_enum(&self) -> bool {
        self.2.is_some()
    }

    fn is_constant(&self) -> bool {
        self.3.is_some()
    }

    fn is_custom_type(&self) -> bool {
        self.4.is_some()
    }
}

impl<'instr> FoundSymbolEither<'instr> for FoundSymbolId<'instr> {
    fn expected_custom_type(&self, span: Span) -> Result<&'instr str, ThrushCompilerError> {
        if let Some(type_id) = self.4 {
            return Ok(type_id);
        }

        Err(ThrushCompilerError::Error(
            String::from("Expected custom type reference"),
            String::from("Expected custom type but found something else."),
            String::default(),
            span,
        ))
    }

    fn expected_constant(&self, span: Span) -> Result<&'instr str, ThrushCompilerError> {
        if let Some(const_id) = self.3 {
            return Ok(const_id);
        }

        Err(ThrushCompilerError::Error(
            String::from("Expected constant reference"),
            String::from("Expected constant but found something else."),
            String::default(),
            span,
        ))
    }

    fn expected_enum(&self, span: Span) -> Result<&'instr str, ThrushCompilerError> {
        if let Some(name) = self.2 {
            return Ok(name);
        }

        Err(ThrushCompilerError::Error(
            String::from("Expected enum reference"),
            String::from("Expected enum but found something else."),
            String::default(),
            span,
        ))
    }

    fn expected_struct(&self, span: Span) -> Result<&'instr str, ThrushCompilerError> {
        if let Some(name) = self.0 {
            return Ok(name);
        }

        Err(ThrushCompilerError::Error(
            String::from("Expected struct reference"),
            String::from("Expected struct but found something else."),
            String::default(),
            span,
        ))
    }

    fn expected_function(&self, span: Span) -> Result<&'instr str, ThrushCompilerError> {
        if let Some(name) = self.1 {
            return Ok(name);
        }

        Err(ThrushCompilerError::Error(
            String::from("Expected function reference"),
            String::from("Expected function but found something else."),
            String::default(),
            span,
        ))
    }

    fn expected_local(&self, span: Span) -> Result<(&'instr str, usize), ThrushCompilerError> {
        if let Some((name, scope_idx)) = self.5 {
            return Ok((name, scope_idx));
        }

        Err(ThrushCompilerError::Error(
            String::from("Expected local reference"),
            String::from("Expected local but found something else."),
            String::default(),
            span,
        ))
    }
}
