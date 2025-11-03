use crate::frontend::parser::contexts::sync::ParserSyncPosition;

#[derive(Debug)]
pub struct ParserControlContext {
    sync_position: Vec<ParserSyncPosition>,

    entry_point: bool,
    global_asm: bool,
    inside_function: bool,
    loop_depth: usize,
    unreacheable_code: usize,
}

impl ParserControlContext {
    #[inline]
    pub fn new() -> Self {
        Self {
            sync_position: Vec::with_capacity(100),
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
    pub fn add_sync_position(&mut self, other: ParserSyncPosition) {
        self.sync_position.push(other);
    }

    #[inline]
    pub fn pop_sync_position(&mut self) {
        self.sync_position.pop();
    }
}

impl ParserControlContext {
    #[inline]
    pub fn set_global_asm(&mut self, value: bool) {
        self.global_asm = value;
    }

    #[inline]
    pub fn set_has_entrypoint(&mut self) {
        self.entry_point = true;
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
    pub fn set_unreacheable_code_scope(&mut self, scope: usize) {
        self.unreacheable_code = scope;
    }
}

impl ParserControlContext {
    #[inline]
    pub fn get_sync_position(&self) -> Option<&ParserSyncPosition> {
        self.sync_position.last()
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
