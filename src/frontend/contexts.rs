use crate::middle::types::Type;

pub struct ParserTypeContext {
    pub function_type: Type,
}

impl ParserTypeContext {
    pub fn new(function_type: Type) -> Self {
        Self { function_type }
    }
}

#[derive(Default)]
pub struct ParserControlContext {
    entry_point: bool,
    rec_structure_ref: bool,
    inside_function: bool,
    inside_loop: bool,
    unreacheable_code: usize,
}

impl ParserControlContext {
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

    pub fn set_is_outside_loop(&mut self) {
        self.inside_loop = false;
    }

    pub fn set_is_inside_function(&mut self) {
        self.inside_function = true;
    }

    pub fn set_is_outside_function(&mut self) {
        self.inside_function = false;
    }

    pub fn set_unreacheable_code_scope(&mut self, scope: usize) {
        self.unreacheable_code = scope;
    }
}
