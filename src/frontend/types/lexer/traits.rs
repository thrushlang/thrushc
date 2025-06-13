use crate::{
    backend::llvm::compiler::context::LLVMCodeGenContext, frontend::types::lexer::ThrushType,
};

pub trait LLVMTypeExtensions {
    fn is_same_size(&self, context: &LLVMCodeGenContext<'_, '_>, other: &ThrushType) -> bool;
}

pub trait ThrushTypeMutableExtensions {
    fn is_mut_fixed_array_type(&self) -> bool;
    fn is_mut_struct_type(&self) -> bool;
    fn is_mut_numeric_type(&self) -> bool;
    fn is_mut_any_nonumeric_type(&self) -> bool;
    fn defer_mut_all(&self) -> ThrushType;
}

pub trait ThrushTypeNumericExtensions {
    fn is_numeric_type(&self) -> bool;
}

pub trait ThrushTypePointerExtensions {
    fn is_typed_ptr(&self) -> bool;
    fn is_all_ptr(&self) -> bool;
    fn is_ptr_struct_type(&self) -> bool;
    fn is_ptr_struct_type_inner(&self) -> bool;
    fn is_ptr_fixed_array_type(&self) -> bool;
    fn is_ptr_fixed_array_type_inner(&self) -> bool;
}
