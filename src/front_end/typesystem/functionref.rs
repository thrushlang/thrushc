use crate::front_end::typesystem::{traits::FunctionReferenceExtensions, types::Type};

impl FunctionReferenceExtensions for Type {
    fn get_fn_ref_type(&self) -> &Type {
        if let Type::Fn(_, kind, ..) = self {
            return kind;
        }

        self
    }
}
