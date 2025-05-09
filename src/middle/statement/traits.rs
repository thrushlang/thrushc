use crate::{
    common::error::ThrushCompilerError,
    frontend::lexer::Span,
    middle::{symbols::types::Bindings, types::Type},
};

use super::{EnumField, EnumFields, StructFields};

pub trait TokenExtensions {
    fn parse_scapes(&self, span: Span) -> Result<Vec<u8>, ThrushCompilerError>;
    fn get_first_byte(&self) -> u8;
}

pub trait EnumFieldsExtensions<'a> {
    fn contain_field(&self, name: &'a str) -> bool;
    fn get_field(&self, name: &'a str) -> EnumField<'a>;
}

pub trait EnumExtensions<'a> {
    fn get_fields(&self) -> EnumFields<'a>;
}

pub trait CustomTypeFieldsExtensions {
    fn get_type(&self) -> Type;
}

pub trait FoundSymbolExtension {
    fn is_custom_type(&self) -> bool;
    fn is_constant(&self) -> bool;
    fn is_structure(&self) -> bool;
    fn is_enum(&self) -> bool;
    fn is_function(&self) -> bool;
}

pub trait StructExtensions<'a> {
    fn contains_field(&self, name: &str) -> bool;
    fn get_field_type(&self, name: &str) -> Option<Type>;
    fn get_fields(&self) -> StructFields<'a>;
    fn get_bindings(&self) -> Bindings<'a>;
}

pub trait FoundSymbolEither<'instr> {
    fn expected_custom_type(&self, span: Span) -> Result<&'instr str, ThrushCompilerError>;
    fn expected_constant(&self, span: Span) -> Result<&'instr str, ThrushCompilerError>;
    fn expected_local(&self, span: Span) -> Result<(&'instr str, usize), ThrushCompilerError>;
    fn expected_function(&self, span: Span) -> Result<&'instr str, ThrushCompilerError>;
    fn expected_enum(&self, span: Span) -> Result<&'instr str, ThrushCompilerError>;
    fn expected_struct(&self, span: Span) -> Result<&'instr str, ThrushCompilerError>;
}

pub trait StructFieldsExtensions {
    fn get_type(&self) -> Type;
}

pub trait ConstructorExtensions {
    fn get_type(&self) -> Type;
}

pub trait AttributesExtensions {
    fn contain_ffi_attribute(&self) -> bool;
    fn contain_ignore_attribute(&self) -> bool;
    fn contain_public_attribute(&self) -> bool;
}
