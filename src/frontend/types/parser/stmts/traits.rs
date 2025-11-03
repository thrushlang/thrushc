use crate::core::errors::standard::ThrushCompilerIssue;

use crate::frontend::lexer::span::Span;
use crate::frontend::lexer::tokentype::TokenType;
use crate::frontend::types::semantic::linter::types::LLVMAttributeComparator;
use crate::frontend::typesystem::modificators::StructureTypeModificator;
use crate::frontend::typesystem::types::Type;

use crate::frontend::types::parser::stmts::types::{EnumField, EnumFields, StructFields};

pub trait TokenExtensions {
    fn scape(&self, span: Span) -> Result<Vec<u8>, ThrushCompilerIssue>;
    fn get_lexeme(&self) -> &str;
    fn get_span(&self) -> Span;
    fn get_type(&self) -> TokenType;
    fn get_ascii_lexeme(&self) -> &str;
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
    fn expected_custom_type(
        &self,
        span: Span,
    ) -> Result<(&'parser str, usize), ThrushCompilerIssue>;
    fn expected_constant(&self, span: Span) -> Result<(&'parser str, usize), ThrushCompilerIssue>;
    fn expected_static(&self, span: Span) -> Result<(&'parser str, usize), ThrushCompilerIssue>;
    fn expected_local(&self, span: Span) -> Result<(&'parser str, usize), ThrushCompilerIssue>;
    fn expected_lli(&self, span: Span) -> Result<(&'parser str, usize), ThrushCompilerIssue>;
    fn expected_function(&self, span: Span) -> Result<&'parser str, ThrushCompilerIssue>;
    fn expected_enum(&self, span: Span) -> Result<(&'parser str, usize), ThrushCompilerIssue>;
    fn expected_struct(&self, span: Span) -> Result<(&'parser str, usize), ThrushCompilerIssue>;
    fn expected_parameter(&self, span: Span) -> Result<&'parser str, ThrushCompilerIssue>;
    fn expected_asm_function(&self, span: Span) -> Result<&'parser str, ThrushCompilerIssue>;
}

pub trait StructFieldsExtensions {
    fn get_type(&self) -> Type;
    fn get_modificator(&self) -> StructureTypeModificator;
}

pub trait ConstructorExtensions {
    fn get_type(&self, name: &str, modificator: StructureTypeModificator) -> Type;
}

pub trait ThrushAttributesExtensions {
    fn has_extern_attribute(&self) -> bool;
    fn has_ignore_attribute(&self) -> bool;
    fn has_public_attribute(&self) -> bool;
    fn has_hot_attr(&self) -> bool;
    fn has_inline_attr(&self) -> bool;
    fn has_noinline_attr(&self) -> bool;
    fn has_minsize_attr(&self) -> bool;
    fn has_inlinealways_attr(&self) -> bool;

    fn has_heap_attr(&self) -> bool;

    fn has_asmalignstack_attribute(&self) -> bool;
    fn has_asmthrow_attribute(&self) -> bool;
    fn has_asmsideffects_attribute(&self) -> bool;

    fn match_attr(&self, cmp: LLVMAttributeComparator) -> Option<Span>;
}
