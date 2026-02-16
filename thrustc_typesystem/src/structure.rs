use thrustc_span::Span;

use crate::{Type, modificators::StructureTypeModificator, traits::TypeStructExtensions};

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
