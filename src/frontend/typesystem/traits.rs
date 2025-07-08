use std::sync::Arc;

use crate::{
    backend::llvm::compiler::context::LLVMCodeGenContext, frontend::typesystem::types::Type,
};

pub trait LLVMTypeExtensions {
    fn llvm_is_same_bit_size(&self, context: &LLVMCodeGenContext<'_, '_>, other: &Type) -> bool;
}

pub trait TypeMutableExtensions {
    fn is_mut_fixed_array_type(&self) -> bool;
    fn is_mut_array_type(&self) -> bool;
    fn is_mut_struct_type(&self) -> bool;
    fn is_mut_numeric_type(&self) -> bool;
}

pub trait TypePointerExtensions {
    fn is_typed_ptr_type(&self) -> bool;
    fn is_all_ptr_type(&self) -> bool;
    fn is_ptr_struct_type(&self) -> bool;
    fn is_ptr_fixed_array_type(&self) -> bool;
}

pub trait TypeStructExtensions {
    fn get_struct_fields(&self) -> &[Arc<Type>];
}

pub trait IndexTypeExtensions {
    fn get_aprox_type(&self, base_depth: usize) -> &Type;
}

pub trait CastTypeExtensions {
    fn narrowing(&self) -> Type;
    fn precompute(&self, other: &Type) -> Type;
}

pub trait DereferenceExtensions {
    fn dereference(&self) -> Type;
    fn dereference_high_level_type(&self) -> Type;
}
