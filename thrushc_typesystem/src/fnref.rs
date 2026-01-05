use crate::Type;
use crate::traits::FunctionReferenceExtensions;

impl FunctionReferenceExtensions for Type {
    fn get_fn_ref_type(&self) -> &Type {
        if let Type::Fn(_, kind, ..) = self {
            return kind;
        }

        self
    }
}
