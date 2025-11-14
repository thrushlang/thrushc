use crate::front_end::typesystem::traits::TypePointerExtensions;
use crate::front_end::typesystem::types::Type;

impl TypePointerExtensions for Type {
    #[inline]
    fn is_ptr_composite_type(&self) -> bool {
        self.is_ptr_struct_type()
    }

    #[inline]
    fn is_ptr_indexable_like_type(&self) -> bool {
        self.is_ptr_struct_type() || self.is_fixed_array_type() || self.is_ptr_array_type()
    }

    #[inline]
    fn is_ptr_aggregate_value_like_type(&self) -> bool {
        self.is_ptr_fixed_array_type()
    }

    #[inline]
    fn is_ptr_aggregate_like_type(&self) -> bool {
        self.is_ptr_fixed_array_type() || self.is_ptr_array_type()
    }

    #[inline]
    fn is_ptr_value_like_type(&self) -> bool {
        self.is_ptr_struct_type()
            || self.is_ptr_fixed_array_type()
            || self.is_ptr_numeric_type()
            || self.is_ptr_array_type()
    }

    #[inline]
    fn is_typed_ptr_type(&self) -> bool {
        if let Type::Ptr(Some(inner), ..) = self {
            return inner.is_typed_ptr_type();
        }

        if let Type::Ptr(None, ..) = self {
            return false;
        }

        true
    }

    #[inline]
    fn is_ptr_struct_type(&self) -> bool {
        if let Type::Ptr(Some(inner), ..) = self {
            return inner.is_struct_type();
        }

        false
    }

    #[inline]
    fn is_ptr_fixed_array_type(&self) -> bool {
        if let Type::Ptr(Some(inner), ..) = self {
            return inner.is_fixed_array_type();
        }

        false
    }

    #[inline]
    fn is_ptr_numeric_type(&self) -> bool {
        if let Type::Ptr(Some(inner), ..) = self {
            return inner.is_numeric_type();
        }

        false
    }

    #[inline]
    fn is_ptr_array_type(&self) -> bool {
        if let Type::Ptr(Some(inner), ..) = self {
            return inner.is_array_type();
        }

        false
    }
}
