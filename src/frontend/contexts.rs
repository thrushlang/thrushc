use crate::middle::types::Type;

#[derive(Debug, Clone, Copy)]
pub enum TypePosition {
    Local,
    Parameter,
    NoRelevant,
}

#[derive(Debug, Clone, Copy)]
pub enum Position {
    Statement,
    Declaration,
    Expression,
    NoRelevant,
}

#[derive(Debug)]
pub struct ParserTypeContext {
    pub function_type: Type,
    pub position: TypePosition,
}

impl ParserTypeContext {
    pub fn new() -> Self {
        Self {
            function_type: Type::Void,
            position: TypePosition::NoRelevant,
        }
    }

    pub fn get_position(&self) -> TypePosition {
        self.position
    }

    pub fn set_position(&mut self, new_position: TypePosition) {
        self.position = new_position;
    }
}

impl TypePosition {
    pub fn is_parameter(&self) -> bool {
        matches!(self, TypePosition::Parameter)
    }
}

pub struct ParserControlContext {
    position: Position,
    entry_point: bool,
    rec_structure_ref: bool,
    inside_function: bool,
    inside_loop: bool,
    unreacheable_code: usize,
}

impl ParserControlContext {
    pub fn new() -> Self {
        Self {
            position: Position::NoRelevant,
            entry_point: false,
            rec_structure_ref: false,
            inside_function: false,
            inside_loop: false,
            unreacheable_code: 0,
        }
    }

    pub fn get_position(&self) -> Position {
        self.position
    }

    pub fn set_position(&mut self, new_position: Position) {
        self.position = new_position;
    }

    pub fn has_entry_point(&self) -> bool {
        self.entry_point
    }

    pub fn is_inside_function(&self) -> bool {
        self.inside_function
    }

    pub fn is_inside_loop(&self) -> bool {
        self.inside_loop
    }

    pub fn get_unreacheable_code_scope(&self) -> usize {
        self.unreacheable_code
    }

    pub fn set_has_entrypoint(&mut self) {
        self.entry_point = true;
    }

    pub fn set_is_inside_loop(&mut self) {
        self.inside_loop = true;
    }

    pub fn set_outside_loop(&mut self) {
        self.inside_loop = false;
    }

    pub fn set_is_inside_function(&mut self) {
        self.inside_function = true;
    }

    pub fn set_outside_function(&mut self) {
        self.inside_function = false;
    }

    pub fn set_unreacheable_code_scope(&mut self, scope: usize) {
        self.unreacheable_code = scope;
    }
}
