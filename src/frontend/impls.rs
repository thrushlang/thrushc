use super::{lexer::Type, objects::Struct, traits::StructureBasics};

impl StructureBasics for Struct<'_> {
    fn contains_field(&self, field_name: &str) -> bool {
        self.iter().any(|field| field.0 == field_name)
    }

    fn get_field_type(&self, field_name: &str) -> Type {
        self.iter()
            .find(|field| field.0 == field_name)
            .map(|field| field.1)
            .unwrap()
    }
}
