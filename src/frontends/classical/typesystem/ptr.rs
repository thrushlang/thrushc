use crate::frontends::classical::typesystem::{traits::TypePointerExtensions, types::Type};

impl TypePointerExtensions for Type {
    #[inline]
    fn is_typed_ptr_type(&self) -> bool {
        if let Type::Ptr(Some(inner)) = self {
            return inner.is_typed_ptr_type();
        }

        if let Type::Ptr(None) = self {
            return false;
        }

        true
    }

    #[inline]
    fn is_ptr_struct_type(&self) -> bool {
        if let Type::Ptr(Some(inner)) = self {
            return inner.is_struct_type();
        }

        false
    }

    #[inline]
    fn is_ptr_fixed_array_type(&self) -> bool {
        if let Type::Ptr(Some(inner)) = self {
            return inner.is_fixed_array_type();
        }

        false
    }
}
