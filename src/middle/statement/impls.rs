use crate::middle::types::Type;

use super::{CustomTypeFields, traits::CustomTypeFieldsExtensions};

use crate::middle::instruction::Instruction;
use crate::middle::statement::traits::{
    AttributesExtensions, ConstructorExtensions, StructFieldsExtensions,
};

use crate::middle::statement::ThrushAttributes;
use crate::middle::statement::{Constructor, StructFields};

impl AttributesExtensions for ThrushAttributes<'_> {
    fn contain_ffi_attribute(&self) -> bool {
        self.iter().any(|attr| attr.is_ffi_attribute())
    }

    fn contain_ignore_attribute(&self) -> bool {
        self.iter().any(|attr| attr.is_ignore_attribute())
    }

    fn contain_public_attribute(&self) -> bool {
        self.iter().any(|attr| attr.is_public_attribute())
    }
}

impl StructFieldsExtensions for StructFields<'_> {
    fn get_type(&self) -> Type {
        let types: Vec<Type> = self.1.iter().map(|field| field.1.clone()).collect();
        Type::create_structure_type(self.0.to_string(), types.as_slice())
    }
}

impl ConstructorExtensions for Constructor<'_> {
    fn get_type(&self) -> Type {
        let types: Vec<Type> = self.1.iter().map(|field| field.2.clone()).collect();
        Type::create_structure_type(self.0.to_string(), types.as_slice())
    }
}

impl CustomTypeFieldsExtensions for CustomTypeFields<'_> {
    fn get_type(&self) -> Type {
        Type::create_structure_type(String::new(), self)
    }
}

impl PartialEq for Instruction<'_> {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Instruction::Integer(..), Instruction::Integer(..))
            | (Instruction::Float(..), Instruction::Float(..))
            | (Instruction::Str(..), Instruction::Str(..)) => true,
            (left, right) => std::mem::discriminant(left) == std::mem::discriminant(right),
        }
    }
}
