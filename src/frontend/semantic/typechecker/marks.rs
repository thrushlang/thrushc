use crate::types::frontend::lexer::types::ThrushType;

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
