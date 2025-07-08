use std::fmt::Display;

use crate::{
    core::errors::{position::CompilationPosition, standard::ThrushCompilerIssue},
    frontend::{
        lexer::span::Span,
        types::{
            ast::metadata::staticvar::StaticMetadata,
            parser::{
                stmts::{
                    traits::{
                        EnumExtensions, EnumFieldsExtensions, FoundSymbolEither,
                        FoundSymbolExtension, StructExtensions,
                    },
                    types::{EnumField, EnumFields, StructFields},
                },
                symbols::{
                    traits::StaticSymbolExtensions,
                    types::{EnumSymbol, StaticSymbol},
                },
            },
        },
        typesystem::types::Type,
    },
};

use super::{
    traits::{
        ConstantSymbolExtensions, FunctionExtensions, LLISymbolExtensions, LocalSymbolExtensions,
    },
    types::{
        ConstantSymbol, FoundSymbolId, Function, LLISymbol, LocalSymbol, ParametersTypes, Struct,
    },
};

impl<'parser> EnumFieldsExtensions<'parser> for EnumFields<'parser> {
    fn contain_field(&self, name: &'parser str) -> bool {
        self.iter().any(|enum_field| enum_field.0 == name)
    }

    fn get_field(&self, name: &'parser str) -> EnumField<'parser> {
        self.iter()
            .find(|enum_field| enum_field.0 == name)
            .cloned()
            .unwrap()
    }
}
impl<'parser> EnumExtensions<'parser> for EnumSymbol<'parser> {
    fn get_fields(&self) -> EnumFields<'parser> {
        self.0.clone()
    }
}

impl LocalSymbolExtensions for LocalSymbol<'_> {
    fn is_mutable(&self) -> bool {
        self.1
    }

    fn get_type(&self) -> Type {
        self.0.clone()
    }
}

impl StaticSymbolExtensions for StaticSymbol<'_> {
    fn get_type(&self) -> Type {
        self.0.clone()
    }

    fn get_metadata(&self) -> StaticMetadata {
        self.1
    }
}

impl ConstantSymbolExtensions for ConstantSymbol<'_> {
    fn get_type(&self) -> Type {
        self.0.clone()
    }
}

impl FunctionExtensions for Function<'_> {
    fn get_type(&self) -> Type {
        self.0.clone()
    }
}

impl LLISymbolExtensions for LLISymbol<'_> {
    fn get_type(&self) -> Type {
        self.0.clone()
    }
}

impl ParametersTypes {
    pub fn new(inner: Vec<Type>) -> Self {
        Self(inner)
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

impl<'parser> StructExtensions<'parser> for Struct<'parser> {
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

    fn get_fields(&self) -> StructFields<'parser> {
        (self.0, self.1.clone())
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

    fn is_static(&self) -> bool {
        self.3.is_some()
    }

    fn is_constant(&self) -> bool {
        self.4.is_some()
    }

    fn is_custom_type(&self) -> bool {
        self.5.is_some()
    }

    fn is_parameter(&self) -> bool {
        self.6.is_some()
    }

    fn is_function_asm(&self) -> bool {
        self.7.is_some()
    }

    fn is_lli(&self) -> bool {
        self.8.is_some()
    }
}

impl<'parser> FoundSymbolEither<'parser> for FoundSymbolId<'parser> {
    fn expected_struct(&self, span: Span) -> Result<&'parser str, ThrushCompilerIssue> {
        if let Some(name) = self.0 {
            return Ok(name);
        }

        Err(ThrushCompilerIssue::Bug(
            String::from("Expected struct reference"),
            String::from("Expected struct but found something else."),
            span,
            CompilationPosition::Parser,
            line!(),
        ))
    }

    fn expected_function(&self, span: Span) -> Result<&'parser str, ThrushCompilerIssue> {
        if let Some(name) = self.1 {
            return Ok(name);
        }

        Err(ThrushCompilerIssue::Bug(
            String::from("Expected function reference"),
            String::from("Expected function but found something else."),
            span,
            CompilationPosition::Parser,
            line!(),
        ))
    }

    fn expected_enum(&self, span: Span) -> Result<&'parser str, ThrushCompilerIssue> {
        if let Some(name) = self.2 {
            return Ok(name);
        }

        Err(ThrushCompilerIssue::Bug(
            String::from("Expected enum reference"),
            String::from("Expected enum but found something else."),
            span,
            CompilationPosition::Parser,
            line!(),
        ))
    }

    fn expected_static(&self, span: Span) -> Result<(&'parser str, usize), ThrushCompilerIssue> {
        if let Some(static_id) = self.3 {
            return Ok(static_id);
        }

        Err(ThrushCompilerIssue::Bug(
            String::from("Expected static reference"),
            String::from("Expected static but found something else."),
            span,
            CompilationPosition::Parser,
            line!(),
        ))
    }

    fn expected_constant(&self, span: Span) -> Result<(&'parser str, usize), ThrushCompilerIssue> {
        if let Some(const_id) = self.4 {
            return Ok(const_id);
        }

        Err(ThrushCompilerIssue::Bug(
            String::from("Expected constant reference"),
            String::from("Expected constant but found something else."),
            span,
            CompilationPosition::Parser,
            line!(),
        ))
    }

    fn expected_custom_type(&self, span: Span) -> Result<&'parser str, ThrushCompilerIssue> {
        if let Some(type_id) = self.5 {
            return Ok(type_id);
        }

        Err(ThrushCompilerIssue::Bug(
            String::from("Expected custom type reference"),
            String::from("Expected custom type but found something else."),
            span,
            CompilationPosition::Parser,
            line!(),
        ))
    }

    fn expected_parameter(&self, span: Span) -> Result<&'parser str, ThrushCompilerIssue> {
        if let Some(name) = self.6 {
            return Ok(name);
        }

        Err(ThrushCompilerIssue::Bug(
            String::from("Expected parameter reference"),
            String::from("Expected parameter but found something else."),
            span,
            CompilationPosition::Parser,
            line!(),
        ))
    }

    fn expected_asm_function(&self, span: Span) -> Result<&'parser str, ThrushCompilerIssue> {
        if let Some(name) = self.7 {
            return Ok(name);
        }

        Err(ThrushCompilerIssue::Bug(
            String::from("Expected assembler function reference"),
            String::from("Expected assembler function but found something else."),
            span,
            CompilationPosition::Parser,
            line!(),
        ))
    }

    fn expected_lli(&self, span: Span) -> Result<(&'parser str, usize), ThrushCompilerIssue> {
        if let Some((name, scope_idx)) = self.8 {
            return Ok((name, scope_idx));
        }

        Err(ThrushCompilerIssue::Bug(
            String::from("Expected low level instruction reference"),
            String::from("Expected LLI but found something else."),
            span,
            CompilationPosition::Parser,
            line!(),
        ))
    }

    fn expected_local(&self, span: Span) -> Result<(&'parser str, usize), ThrushCompilerIssue> {
        if let Some((name, scope_idx)) = self.9 {
            return Ok((name, scope_idx));
        }

        Err(ThrushCompilerIssue::Bug(
            String::from("Expected local reference"),
            String::from("Expected local but found something else."),
            span,
            CompilationPosition::Parser,
            line!(),
        ))
    }
}
