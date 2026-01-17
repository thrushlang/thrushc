use thrushc_errors::CompilationIssue;
use thrushc_span::Span;
use thrushc_typesystem::{Type, modificators::StructureTypeModificator};

pub trait AstGetType {
    fn get_any_type(&self) -> Result<&Type, CompilationIssue>;
    fn get_value_type(&self) -> Result<&Type, CompilationIssue>;
}

pub trait AstCodeLocation {
    fn get_span(&self) -> Span;
}

pub trait AstStatementExtentions {
    fn is_statement(&self) -> bool;
}

pub trait AstStandardExtensions {
    fn is_literal_value(&self) -> bool;
    fn is_reference(&self) -> bool;
    fn is_unreacheable(&self) -> bool;
    fn is_before_unary(&self) -> bool;
    fn is_import(&self) -> bool;
    fn is_function(&self) -> bool;
    fn is_intrinsic(&self) -> bool;
    fn is_asm_function(&self) -> bool;
    fn is_global_asm(&self) -> bool;
    fn is_struct(&self) -> bool;
    fn is_enum(&self) -> bool;
    fn is_str(&self) -> bool;
    fn is_constant(&self) -> bool;
    fn is_static(&self) -> bool;
    fn is_integer(&self) -> bool;
    fn is_terminator(&self) -> bool;
    fn is_custom_type(&self) -> bool;
    fn is_break(&self) -> bool;
    fn is_continue(&self) -> bool;
    fn is_conditional(&self) -> bool;
}

pub trait AstCodeBlockEntensions {
    fn is_empty_block(&self) -> bool;
    fn has_terminator(&self) -> bool;
}

pub trait AstMemoryExtensions {
    fn is_allocated(&self) -> bool;
    fn is_allocated_value(&self) -> Result<bool, CompilationIssue>;
}

pub trait AstConstantExtensions {
    fn is_constant_value(&self) -> bool;
}

pub trait AstMutabilityExtensions {
    fn is_mutable(&self) -> bool;
}

pub trait AstScopeExtensions {
    fn is_compatible_with_main_scope(&self) -> bool;
}

pub trait AstStructureDataExtensions<'parser> {
    fn new(name: &'parser str, modificator: StructureTypeModificator, span: Span) -> Self;
}
