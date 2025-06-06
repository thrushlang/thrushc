#![allow(clippy::upper_case_acronyms)]

use crate::frontend::types::lexer::ThrushType;

#[derive(Debug, Default, Clone, Copy)]
pub enum TypeCheckerTypePosition {
    Function,

    #[default]
    None,
}

#[derive(Debug)]
pub struct TypeCheckerTypeContext<'types> {
    current_function_type: &'types ThrushType,
    position: TypeCheckerTypePosition,
}

impl<'types> TypeCheckerTypeContext<'types> {
    pub fn new() -> Self {
        Self {
            current_function_type: &ThrushType::Void,
            position: TypeCheckerTypePosition::default(),
        }
    }

    pub fn set_function_type(&mut self, new_type: &'types ThrushType) {
        self.current_function_type = new_type;
    }

    pub fn set_type_position(&mut self, new_position: TypeCheckerTypePosition) {
        self.position = new_position;
    }

    pub fn get_function_type(&self) -> &ThrushType {
        self.current_function_type
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub enum TypeCheckerTypeCheckSource {
    Local,
    Call,
    LLI,

    #[default]
    NoRelevant,
}

impl TypeCheckerTypeCheckSource {
    pub fn is_local(&self) -> bool {
        matches!(self, TypeCheckerTypeCheckSource::Local)
    }

    pub fn is_lli(&self) -> bool {
        matches!(self, TypeCheckerTypeCheckSource::LLI)
    }
}
