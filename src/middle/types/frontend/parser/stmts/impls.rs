use std::fmt::Display;

use crate::middle::types::frontend::lexer::types::ThrushType;

use super::{
    instruction::Instruction,
    traits::{
        CompilerAttributesExtensions, ConstructorExtensions, CustomTypeFieldsExtensions,
        StructFieldsExtensions,
    },
    types::{CompilerAttributes, Constructor, CustomTypeFields, StructFields},
};

impl CompilerAttributesExtensions for CompilerAttributes<'_> {
    fn has_ffi_attribute(&self) -> bool {
        self.iter().any(|attr| attr.is_ffi_attribute())
    }

    fn has_ignore_attribute(&self) -> bool {
        self.iter().any(|attr| attr.is_ignore_attribute())
    }

    fn has_public_attribute(&self) -> bool {
        self.iter().any(|attr| attr.is_public_attribute())
    }
}

impl StructFieldsExtensions for StructFields<'_> {
    fn get_type(&self) -> ThrushType {
        let types: Vec<ThrushType> = self.1.iter().map(|field| field.1.clone()).collect();
        ThrushType::create_structure_type(self.0.to_string(), types.as_slice())
    }
}

impl ConstructorExtensions for Constructor<'_> {
    fn get_type(&self) -> ThrushType {
        let types: Vec<ThrushType> = self.1.iter().map(|field| field.2.clone()).collect();
        ThrushType::create_structure_type(self.0.to_string(), types.as_slice())
    }
}

impl CustomTypeFieldsExtensions for CustomTypeFields<'_> {
    fn get_type(&self) -> ThrushType {
        ThrushType::create_structure_type(String::new(), self)
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

impl Display for Instruction<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Instruction::Null => write!(f, "null"),
            Instruction::Pass { .. } => write!(f, "pass"),
            Instruction::Integer(_, value, ..) => write!(f, "{}", value),
            Instruction::Float(_, value, ..) => write!(f, "{}", value),
            Instruction::Boolean(_, value, ..) => write!(f, "{}", value),
            Instruction::Str(_, value, ..) => {
                write!(f, "\"{}\"", String::from_utf8_lossy(value))
            }
            Instruction::Block { stmts, .. } => {
                let _ = writeln!(f, "{{");

                for stmt in stmts {
                    let _ = write!(f, "{}", stmt);
                }

                let _ = writeln!(f, "}}");

                Ok(())
            }

            _ => unreachable!(),
        }
    }
}
