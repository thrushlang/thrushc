use thrushc_span::Span;

use crate::Type;
use crate::modificators::StructureTypeModificator;

pub trait TypeIsExtensions {
    fn is_char_type(&self) -> bool;
    fn is_void_type(&self) -> bool;
    fn is_bool_type(&self) -> bool;
    fn is_struct_type(&self) -> bool;
    fn is_fixed_array_type(&self) -> bool;
    fn is_array_type(&self) -> bool;
    fn is_float_type(&self) -> bool;
    fn is_ptr_type(&self) -> bool;
    fn is_ptr_like_type(&self) -> bool;
    fn is_address_type(&self) -> bool;
    fn is_const_type(&self) -> bool;
    fn is_fnref_type(&self) -> bool;
    fn is_numeric_type(&self) -> bool;
    fn is_unsigned_integer_type(&self) -> bool;
    fn is_signed_integer_type(&self) -> bool;
    fn is_lesseq_unsigned32bit_integer(&self) -> bool;
    fn is_integer_type(&self) -> bool;
}

pub trait FunctionReferenceExtensions {
    fn get_fn_ref_type(&self) -> &Type;
}

pub trait IndexExtensions {
    fn calculate_index_type(&self, depth: usize) -> &Type;
}

pub trait TypeExtensions {
    fn get_type_with_depth(&self, base_depth: usize) -> &Type;
    fn get_type_ref(&self) -> Type;
    fn is_value(&self) -> bool;
    fn is_const_value(&self) -> bool;
}

pub trait TypeFixedArrayEntensions {
    fn get_fixed_array_base_type(&self) -> Type;
    fn get_fixed_array_type_herarchy(&self) -> u8;
}

pub trait TypeArrayEntensions {
    fn get_array_base_type(&self) -> Type;
    fn get_array_type_herarchy(&self) -> u8;
}

pub trait TypePointerExtensions {
    fn is_ptr_composite_type(&self) -> bool;
    fn is_ptr_aggregate_value_like_type(&self) -> bool;
    fn is_ptr_aggregate_like_type(&self) -> bool;
    fn is_ptr_indexable_like_type(&self) -> bool;
    fn is_ptr_value_like_type(&self) -> bool;
    fn is_typed_ptr_type(&self) -> bool;

    fn is_ptr_struct_type(&self) -> bool;
    fn is_ptr_fixed_array_type(&self) -> bool;
    fn is_ptr_array_type(&self) -> bool;
    fn is_ptr_numeric_type(&self) -> bool;
}

pub trait TypeStructExtensions {
    fn get_struct_fields(&self) -> &[Type];
    fn create_struct_type(
        name: String,
        fields: &[Type],
        modificator: StructureTypeModificator,
        span: Span,
    ) -> Type;
}

pub trait CastTypeExtensions {
    fn narrowing(&self) -> Type;
}

pub trait TypeCodeLocation {
    fn get_span(&self) -> Span;
}

pub trait DereferenceExtensions {
    fn dereference(&self) -> Type;
}

pub trait InfererTypeExtensions {
    fn inferer_inner_type_from_type(&mut self, other: &Type);
    fn has_inferer_inner_type(&self) -> bool;
    fn is_inferer_inner_type_valid(&self) -> bool;
    fn get_inferer_inner_type(&self) -> Type;
}
