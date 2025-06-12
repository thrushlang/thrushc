use crate::frontend::types::lexer::ThrushType;

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
    inside_bind: bool,
    inside_loop: bool,
    unreacheable_code: usize,
}

#[derive(Debug)]
pub struct ParserTypeContext {
    function_type: ThrushType,
}

impl ParserTypeContext {
    pub fn new() -> Self {
        Self {
            function_type: ThrushType::Void,
        }
    }

    pub fn set_function_type(&mut self, new_type: ThrushType) {
        self.function_type = new_type;
    }

    pub fn get_function_type(&self) -> ThrushType {
        self.function_type.clone()
    }
}

impl ParserControlContext {
    pub fn new() -> Self {
        Self {
            sync_position: SyncPosition::NoRelevant,
            entry_point: false,
            inside_function: false,
            inside_bind: false,
            inside_loop: false,
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

    pub fn set_inside_loop(&mut self, value: bool) {
        self.inside_loop = value;
    }

    pub fn get_inside_loop(&self) -> bool {
        self.inside_loop
    }

    pub fn set_inside_bind(&mut self, value: bool) {
        self.inside_bind = value;
    }

    pub fn get_inside_bind(&self) -> bool {
        self.inside_bind
    }

    pub fn get_unreacheable_code_scope(&self) -> usize {
        self.unreacheable_code
    }

    pub fn set_unreacheable_code_scope(&mut self, scope: usize) {
        self.unreacheable_code = scope;
    }
}
