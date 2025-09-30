use crate::frontends::classical::typesystem::{traits::TypeMutableExtensions, types::Type};

impl TypeMutableExtensions for Type {
    #[inline]
    fn is_mut_fixed_array_type(&self) -> bool {
        if let Type::Mut(inner) = self {
            return inner.is_fixed_array_type();
        }

        false
    }

    #[inline]
    fn is_mut_array_type(&self) -> bool {
        if let Type::Mut(inner) = self {
            return inner.is_array_type();
        }

        false
    }

    #[inline]
    fn is_mut_struct_type(&self) -> bool {
        if let Type::Mut(inner) = self {
            return inner.is_struct_type();
        }

        false
    }
}
