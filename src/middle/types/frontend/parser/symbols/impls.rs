use std::fmt::Display;

use crate::{
    frontend::lexer::Span,
    middle::types::frontend::{
        lexer::types::ThrushType,
        parser::stmts::{
            traits::{FoundSymbolEither, FoundSymbolExtension, StructExtensions},
            types::StructFields,
        },
    },
    standard::error::ThrushCompilerIssue,
};

use super::{
    traits::{
        BindExtensions, BindingsExtensions, ConstantSymbolExtensions, FunctionExtensions,
        LocalSymbolExtensions,
    },
    types::{
        Bind, Bindings, ConstantSymbol, FoundSymbolId, Function, LocalSymbol, ParametersTypes,
        Struct,
    },
};

impl LocalSymbolExtensions for LocalSymbol<'_> {
    fn is_undefined(&self) -> bool {
        self.2
    }

    fn is_mutable(&self) -> bool {
        self.1
    }

    fn get_span(&self) -> Span {
        self.3
    }

    fn get_type(&self) -> ThrushType {
        self.0.clone()
    }
}

impl ConstantSymbolExtensions for ConstantSymbol<'_> {
    fn get_type(&self) -> ThrushType {
        self.0.clone()
    }
}

impl FunctionExtensions for Function<'_> {
    fn ignore_more_args(&self) -> bool {
        self.2
    }

    fn get_type(&self) -> ThrushType {
        self.0.clone()
    }

    fn get_parameters_size(&self) -> usize {
        self.1.get_size()
    }

    fn get_parameters_types(&self) -> &ParametersTypes {
        &self.1
    }
}

impl ParametersTypes {
    pub fn new(inner: Vec<ThrushType>) -> Self {
        Self(inner)
    }

    pub fn get_size(&self) -> usize {
        self.0.len()
    }
}

impl Display for ParametersTypes {
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

    fn get_parameters_types(&self) -> &[ThrushType] {
        &self.2
    }

    fn get_type(&self) -> ThrushType {
        self.1.clone()
    }
}

impl<'a> StructExtensions<'a> for Struct<'a> {
    fn contains_field(&self, name: &str) -> bool {
        self.1.iter().any(|field| field.0 == name)
    }

    fn get_field_type(&self, name: &str) -> Option<ThrushType> {
        if let Some(field) = self.1.iter().find(|field| field.0 == name) {
            let field_type: ThrushType = field.1.clone();
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

    fn is_parameter(&self) -> bool {
        self.5.is_some()
    }
}

impl<'instr> FoundSymbolEither<'instr> for FoundSymbolId<'instr> {
    fn expected_custom_type(&self, span: Span) -> Result<&'instr str, ThrushCompilerIssue> {
        if let Some(type_id) = self.4 {
            return Ok(type_id);
        }

        Err(ThrushCompilerIssue::Error(
            String::from("Expected custom type reference"),
            String::from("Expected custom type but found something else."),
            None,
            span,
        ))
    }

    fn expected_constant(&self, span: Span) -> Result<&'instr str, ThrushCompilerIssue> {
        if let Some(const_id) = self.3 {
            return Ok(const_id);
        }

        Err(ThrushCompilerIssue::Error(
            String::from("Expected constant reference"),
            String::from("Expected constant but found something else."),
            None,
            span,
        ))
    }

    fn expected_enum(&self, span: Span) -> Result<&'instr str, ThrushCompilerIssue> {
        if let Some(name) = self.2 {
            return Ok(name);
        }

        Err(ThrushCompilerIssue::Error(
            String::from("Expected enum reference"),
            String::from("Expected enum but found something else."),
            None,
            span,
        ))
    }

    fn expected_struct(&self, span: Span) -> Result<&'instr str, ThrushCompilerIssue> {
        if let Some(name) = self.0 {
            return Ok(name);
        }

        Err(ThrushCompilerIssue::Error(
            String::from("Expected struct reference"),
            String::from("Expected struct but found something else."),
            None,
            span,
        ))
    }

    fn expected_function(&self, span: Span) -> Result<&'instr str, ThrushCompilerIssue> {
        if let Some(name) = self.1 {
            return Ok(name);
        }

        Err(ThrushCompilerIssue::Error(
            String::from("Expected function reference"),
            String::from("Expected function but found something else."),
            None,
            span,
        ))
    }

    fn expected_parameter(&self, span: Span) -> Result<&'instr str, ThrushCompilerIssue> {
        if let Some(name) = self.5 {
            return Ok(name);
        }

        Err(ThrushCompilerIssue::Error(
            String::from("Expected parameter reference"),
            String::from("Expected parameter but found something else."),
            None,
            span,
        ))
    }

    fn expected_local(&self, span: Span) -> Result<(&'instr str, usize), ThrushCompilerIssue> {
        if let Some((name, scope_idx)) = self.6 {
            return Ok((name, scope_idx));
        }

        Err(ThrushCompilerIssue::Error(
            String::from("Expected local reference"),
            String::from("Expected local but found something else."),
            None,
            span,
        ))
    }
}
