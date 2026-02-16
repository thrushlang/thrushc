use crate::Type;
use crate::traits::FunctionReferenceExtensions;

impl FunctionReferenceExtensions for Type {
    fn get_function_reference_return_type(&self) -> Type {
        if let Type::Fn(_, kind, ..) = self {
            return (**kind).clone();
        }

        self.clone()
    }
}
