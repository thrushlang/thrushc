use crate::core::diagnostic::span::Span;
use crate::core::errors::standard::CompilationIssue;

use crate::front_end::lexer::tokentype::TokenType;
use crate::front_end::typesystem::modificators::StructureTypeModificator;
use crate::front_end::typesystem::types::Type;

use crate::front_end::types::parser::stmts::types::EnumField;
use crate::front_end::types::parser::stmts::types::EnumFields;
use crate::front_end::types::parser::stmts::types::StructFields;

pub trait TokenExtensions {
    fn get_lexeme(&self) -> &str;
    fn get_span(&self) -> Span;
    fn get_type(&self) -> TokenType;
    fn get_ascii_lexeme(&self) -> &str;
    fn get_bytes_lexeme(&self) -> &[u8];
    fn get_lexeme_first_byte(&self) -> u64;
}

pub trait EnumFieldsExtensions<'parser> {
    fn contain_field(&self, name: &'parser str) -> bool;
    fn get_field(&self, name: &'parser str) -> EnumField<'parser>;
}

pub trait EnumExtensions<'parser> {
    fn get_fields(&self) -> EnumFields<'parser>;
}

pub trait FoundSymbolExtension {
    fn is_custom_type(&self) -> bool;
    fn is_function(&self) -> bool;
    fn is_static(&self) -> bool;
    fn is_constant(&self) -> bool;
    fn is_structure(&self) -> bool;
    fn is_function_asm(&self) -> bool;
    fn is_parameter(&self) -> bool;
    fn is_intrinsic(&self) -> bool;
    fn is_lli(&self) -> bool;
    fn is_local(&self) -> bool;
}

pub trait StructExtensions<'parser> {
    fn contains_field(&self, name: &str) -> bool;

    fn get_field_type(&self, name: &str) -> Option<Type>;
    fn get_fields(&self) -> StructFields<'parser>;
    fn get_modificator(&self) -> StructureTypeModificator;
}

pub trait FoundSymbolEither<'parser> {
    fn expected_custom_type(&self, span: Span) -> Result<(&'parser str, usize), CompilationIssue>;
    fn expected_constant(&self, span: Span) -> Result<(&'parser str, usize), CompilationIssue>;
    fn expected_static(&self, span: Span) -> Result<(&'parser str, usize), CompilationIssue>;
    fn expected_local(&self, span: Span) -> Result<(&'parser str, usize), CompilationIssue>;
    fn expected_lli(&self, span: Span) -> Result<(&'parser str, usize), CompilationIssue>;
    fn expected_function(&self, span: Span) -> Result<&'parser str, CompilationIssue>;
    fn expected_intrinsic(&self, span: Span) -> Result<&'parser str, CompilationIssue>;
    fn expected_enum(&self, span: Span) -> Result<(&'parser str, usize), CompilationIssue>;
    fn expected_struct(&self, span: Span) -> Result<(&'parser str, usize), CompilationIssue>;
    fn expected_parameter(&self, span: Span) -> Result<&'parser str, CompilationIssue>;
    fn expected_asm_function(&self, span: Span) -> Result<&'parser str, CompilationIssue>;
}

pub trait StructFieldsExtensions {
    fn get_type(&self) -> Type;
    fn get_modificator(&self) -> StructureTypeModificator;
}

pub trait ConstructorExtensions {
    fn get_type(&self, name: &str, modificator: StructureTypeModificator) -> Type;
}
