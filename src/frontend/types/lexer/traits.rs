use crate::{
    backend::llvm::compiler::context::LLVMCodeGenContext,
    core::errors::standard::ThrushCompilerIssue,
    frontend::{lexer::span::Span, types::lexer::ThrushType},
};

pub trait LLVMTypeExtensions {
    fn is_same_size(&self, context: &LLVMCodeGenContext<'_, '_>, other: &ThrushType) -> bool;
}

pub trait ThrushTypeStructTypeExtensions {
    fn parser_get_struct_name(&self, span: Span) -> Result<String, ThrushCompilerIssue>;
}

pub trait ThrushTypeMutableExtensions {
    fn is_mut_struct_type(&self) -> bool;
    fn is_mut_array_type(&self) -> bool;
    fn is_mut_numeric_type(&self) -> bool;
    fn defer_mut_all(&self) -> ThrushType;
}

pub trait ThrushTypeNumericExtensions {
    fn is_numeric_type(&self) -> bool;
}

pub trait ThrushTypePointerExtensions {
    fn is_typed_ptr(&self) -> bool;
    fn is_ptr_struct_type(&self) -> bool;
    fn is_ptr_array_type(&self) -> bool;
}
