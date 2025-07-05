use std::sync::Arc;

use crate::{backend::llvm::compiler::context::LLVMCodeGenContext, frontend::types::lexer::Type};

pub trait LLVMTypeExtensions {
    fn is_same_size(&self, context: &LLVMCodeGenContext<'_, '_>, other: &Type) -> bool;
}

pub trait TypeMutableExtensions {
    fn is_mut_fixed_array_type(&self) -> bool;
    fn is_mut_array_type(&self) -> bool;
    fn is_mut_struct_type(&self) -> bool;
    fn is_mut_numeric_type(&self) -> bool;
    fn defer_mut_all(&self) -> Type;
}

pub trait TypePointerExtensions {
    fn is_typed_ptr(&self) -> bool;
    fn is_all_ptr(&self) -> bool;
    fn is_ptr_struct_type(&self) -> bool;
    fn is_ptr_fixed_array_type(&self) -> bool;
}

pub trait TypeStructExtensions {
    fn get_struct_fields(&self) -> &[Arc<Type>];
}
