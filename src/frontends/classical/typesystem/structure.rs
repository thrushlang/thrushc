use crate::frontends::classical::typesystem::{
    modificators::StructureTypeModificator, traits::TypeStructExtensions, types::Type,
};

impl TypeStructExtensions for Type {
    #[inline]
    fn create_struct_type(
        name: String,
        fields: &[Type],
        modificator: StructureTypeModificator,
    ) -> Type {
        Type::Struct(name, fields.to_vec(), modificator)
    }

    #[inline]
    fn get_struct_fields(&self) -> &[Type] {
        if let Type::Struct(_, fields, ..) = self {
            return fields;
        }

        &[]
    }
}
