use crate::core::diagnostic::span::Span;
use crate::front_end::typesystem::modificators::StructureTypeModificator;
use crate::front_end::typesystem::traits::TypeStructExtensions;
use crate::front_end::typesystem::types::Type;

impl TypeStructExtensions for Type {
    #[inline]
    fn create_struct_type(
        name: String,
        fields: &[Type],
        modificator: StructureTypeModificator,
        span: Span,
    ) -> Type {
        Type::Struct(name, fields.to_vec(), modificator, span)
    }

    #[inline]
    fn get_struct_fields(&self) -> &[Type] {
        if let Type::Struct(_, fields, ..) = self {
            return fields;
        }

        &[]
    }
}
