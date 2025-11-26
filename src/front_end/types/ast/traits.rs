use crate::core::errors::standard::ThrushCompilerIssue;

pub trait AstStatementExtentions {
    fn is_statement(&self) -> bool;
}

pub trait AstStandardExtensions {
    fn is_literal_value(&self) -> bool;
    fn is_reference(&self) -> bool;
    fn is_before_unary(&self) -> bool;
    fn is_function(&self) -> bool;
    fn is_intrinsic(&self) -> bool;
    fn is_asm_function(&self) -> bool;
    fn is_struct(&self) -> bool;
    fn is_enum(&self) -> bool;
    fn is_str(&self) -> bool;
    fn is_constant(&self) -> bool;
    fn is_static(&self) -> bool;
    fn is_integer(&self) -> bool;
    fn is_terminator(&self) -> bool;
    fn is_break(&self) -> bool;
    fn is_continue(&self) -> bool;
    fn is_lli(&self) -> bool;
}

pub trait AstCodeBlockEntensions {
    fn is_empty_block(&self) -> bool;
    fn has_terminator(&self) -> bool;
}

pub trait AstMemoryExtensions {
    fn is_allocated(&self) -> bool;
    fn is_allocated_value(&self) -> Result<bool, ThrushCompilerIssue>;
}

pub trait AstConstantExtensions {
    fn is_constant_value(&self) -> bool;
}

pub trait AstMutabilityExtensions {
    fn is_mutable(&self) -> bool;
}
