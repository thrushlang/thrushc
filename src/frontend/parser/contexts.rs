use crate::frontend::types::lexer::Type;

#[derive(Debug, Clone, Copy)]
pub enum SyncPosition {
    Statement,
    Declaration,
    Expression,
    NoRelevant,
}

#[derive(Debug)]
pub struct ParserControlContext {
    sync_position: SyncPosition,

    entry_point: bool,
    inside_function: bool,
    loop_depth: usize,
    unreacheable_code: usize,
}

#[derive(Debug)]
pub struct ParserTypeContext {
    function_type: Type,
}

impl ParserTypeContext {
    pub fn new() -> Self {
        Self {
            function_type: Type::Void,
        }
    }

    pub fn set_function_type(&mut self, new_type: Type) {
        self.function_type = new_type;
    }

    pub fn get_function_type(&self) -> Type {
        self.function_type.clone()
    }
}

impl ParserControlContext {
    pub fn new() -> Self {
        Self {
            sync_position: SyncPosition::NoRelevant,
            entry_point: false,
            inside_function: false,
            loop_depth: 0,
            unreacheable_code: 0,
        }
    }

    pub fn get_sync_position(&self) -> SyncPosition {
        self.sync_position
    }

    pub fn set_sync_position(&mut self, new_sync_position: SyncPosition) {
        self.sync_position = new_sync_position;
    }

    pub fn set_entrypoint(&mut self, value: bool) {
        self.entry_point = value;
    }

    pub fn get_entrypoint(&self) -> bool {
        self.entry_point
    }

    pub fn set_inside_function(&mut self, value: bool) {
        self.inside_function = value;
    }

    pub fn get_inside_function(&self) -> bool {
        self.inside_function
    }

    pub fn increment_loop_depth(&mut self) {
        self.loop_depth += 1;
    }

    pub fn decrement_loop_depth(&mut self) {
        self.loop_depth -= 1;
    }

    pub fn reset_loop_depth(&mut self) {
        self.loop_depth = 0;
    }

    pub fn is_inside_loop(&self) -> bool {
        self.loop_depth > 0
    }

    pub fn get_unreacheable_code_scope(&self) -> usize {
        self.unreacheable_code
    }

    pub fn set_unreacheable_code_scope(&mut self, scope: usize) {
        self.unreacheable_code = scope;
    }
}
