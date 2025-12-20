use crate::front_end::typesystem::traits::DereferenceExtensions;
use crate::front_end::typesystem::types::Type;

impl DereferenceExtensions for Type {
    fn dereference(&self) -> Type {
        if let Type::Ptr(Some(any), ..) = self {
            return (**any).clone();
        }

        self.clone()
    }
}
