use crate::frontends::classical::typesystem::{traits::DereferenceExtensions, types::Type};

impl DereferenceExtensions for Type {
    fn dereference(&self) -> Type {
        if let Type::Ptr(Some(any)) = self {
            return (**any).clone();
        }

        if let Type::Const(any) = self {
            return (**any).clone();
        }

        self.clone()
    }
}
