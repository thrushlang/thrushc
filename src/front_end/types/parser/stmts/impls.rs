use crate::front_end::types::parser::stmts::traits::ConstructorExtensions;
use crate::front_end::types::parser::stmts::traits::StructFieldsExtensions;
use crate::front_end::types::parser::stmts::types::Constructor;
use crate::front_end::types::parser::stmts::types::StructFields;
use crate::front_end::typesystem::modificators::StructureTypeModificator;
use crate::front_end::typesystem::traits::TypeStructExtensions;
use crate::front_end::typesystem::types::Type;

impl StructFieldsExtensions for StructFields<'_> {
    #[inline]
    fn get_type(&self) -> Type {
        let types: Vec<Type> = self.1.iter().map(|field| field.1.clone()).collect();
        Type::create_struct_type(self.0.to_string(), types.as_slice(), self.get_modificator())
    }

    #[inline]
    fn get_modificator(&self) -> StructureTypeModificator {
        self.2
    }
}

impl ConstructorExtensions for Constructor<'_> {
    #[inline]
    fn get_type(&self, name: &str, modificator: StructureTypeModificator) -> Type {
        let types: Vec<Type> = self.iter().map(|field| field.2.clone()).collect();
        Type::create_struct_type(name.to_string(), types.as_slice(), modificator)
    }
}
