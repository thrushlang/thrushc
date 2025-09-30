use crate::frontends::classical::typesystem::{traits::DereferenceExtensions, types::Type};

impl DereferenceExtensions for Type {
    fn dereference(&self) -> Type {
        if let Type::Ptr(Some(any)) = self {
            return (**any).clone();
        }

        if let Type::Mut(any) = self {
            return (**any).clone();
        }

        if let Type::Const(any) = self {
            return (**any).clone();
        }

        self.clone()
    }

    fn dereference_high_level_type(&self) -> Type {
        if let Type::Mut(inner_type) = self {
            return (**inner_type).clone();
        }

        if let Type::Const(inner_type) = self {
            return inner_type.dereference_high_level_type();
        }

        self.clone()
    }
}
