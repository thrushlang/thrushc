use crate::{
    backends::classical::llvm::compiler::context::LLVMCodeGenContext,
    frontends::classical::typesystem::{modificators::StructureTypeModificator, types::Type},
};

pub trait LLVMTypeExtensions {
    fn llvm_is_same_bit_size(&self, context: &LLVMCodeGenContext<'_, '_>, other: &Type) -> bool;
    fn llvm_is_ptr_type(&self) -> bool;
    fn llvm_is_int_type(&self) -> bool;
    fn llvm_is_float_type(&self) -> bool;
}

pub trait TypeExtensions {
    fn get_type_with_depth(&self, base_depth: usize) -> &Type;
    fn get_type_fn_ref(&self) -> &Type;
}

pub trait TypeMutableExtensions {
    fn is_mut_fixed_array_type(&self) -> bool;
    fn is_mut_array_type(&self) -> bool;
    fn is_mut_struct_type(&self) -> bool;
}

pub trait TypeFixedArrayEntensions {
    fn get_farray_base_type(&self) -> &Type;
}

pub trait TypeArrayEntensions {
    fn get_array_base_type(&self) -> &Type;
    fn get_array_type_herarchy(&self) -> u8;
}

pub trait TypePointerExtensions {
    fn is_typed_ptr_type(&self) -> bool;
    fn is_ptr_struct_type(&self) -> bool;
    fn is_ptr_fixed_array_type(&self) -> bool;
}

pub trait TypeStructExtensions {
    fn get_struct_fields(&self) -> &[Type];
    fn create_struct_type(
        name: String,
        fields: &[Type],
        modificator: StructureTypeModificator,
    ) -> Type;
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
