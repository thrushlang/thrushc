use crate::frontend::typesystem::{traits::TypeFixedArrayEntensions, types::Type};

impl TypeFixedArrayEntensions for Type {
    #[inline]
    fn get_farray_base_type(&self) -> &Type {
        if let Type::FixedArray(inner, ..) = self {
            return inner;
        }

        if let Type::Ptr(Some(inner)) = self {
            return inner.get_farray_base_type();
        }

        if let Type::Const(inner) = self {
            return inner.get_farray_base_type();
        }

        self
    }
}
