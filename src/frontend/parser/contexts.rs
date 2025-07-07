use crate::frontend::typesystem::types::Type;

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
    global_asm: bool,
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

    #[inline]
    pub fn set_function_type(&mut self, new_type: Type) {
        self.function_type = new_type;
    }

    #[inline]
    pub fn get_function_type(&self) -> Type {
        self.function_type.clone()
    }
}

impl ParserControlContext {
    pub fn new() -> Self {
        Self {
            sync_position: SyncPosition::NoRelevant,
            entry_point: false,
            global_asm: false,
            inside_function: false,
            loop_depth: 0,
            unreacheable_code: 0,
        }
    }
}

impl ParserControlContext {
    #[inline]
    pub fn set_sync_position(&mut self, new_sync_position: SyncPosition) {
        self.sync_position = new_sync_position;
    }

    #[inline]
    pub fn set_global_asm(&mut self, value: bool) {
        self.global_asm = value;
    }

    #[inline]
    pub fn set_entrypoint(&mut self, value: bool) {
        self.entry_point = value;
    }

    #[inline]
    pub fn set_inside_function(&mut self, value: bool) {
        self.inside_function = value;
    }

    #[inline]
    pub fn increment_loop_depth(&mut self) {
        self.loop_depth += 1;
    }

    #[inline]
    pub fn decrement_loop_depth(&mut self) {
        self.loop_depth -= 1;
    }

    #[inline]
    pub fn reset_loop_depth(&mut self) {
        self.loop_depth = 0;
    }

    #[inline]
    pub fn set_unreacheable_code_scope(&mut self, scope: usize) {
        self.unreacheable_code = scope;
    }
}

impl ParserControlContext {
    #[inline]
    pub fn get_sync_position(&self) -> SyncPosition {
        self.sync_position
    }

    #[inline]
    pub fn get_global_asm(&self) -> bool {
        self.global_asm
    }

    #[inline]
    pub fn get_entrypoint(&self) -> bool {
        self.entry_point
    }

    #[inline]
    pub fn get_inside_function(&self) -> bool {
        self.inside_function
    }

    #[inline]
    pub fn is_inside_loop(&self) -> bool {
        self.loop_depth > 0
    }

    #[inline]
    pub fn get_unreacheable_code_scope(&self) -> usize {
        self.unreacheable_code
    }
}
