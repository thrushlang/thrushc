use thrustc_typesystem::Type;

#[derive(Debug, Clone, Copy, Default)]
pub enum ParserSyncPosition {
    Statement,
    Declaration,
    Expression,

    #[default]
    NoRelevant,
}

#[derive(Debug)]
pub struct ParserControlContext {
    sync_position: Vec<ParserSyncPosition>,
    expression_depth: u32,
}

impl ParserControlContext {
    #[inline]
    pub fn new() -> Self {
        Self {
            sync_position: Vec::with_capacity(u8::MAX as usize),
            expression_depth: 0,
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

    #[inline]
    pub fn reset_sync_position(&mut self) {
        self.sync_position.clear();
    }

    #[inline]
    pub fn increase_expression_depth(&mut self) {
        self.expression_depth = self.expression_depth.saturating_add(1);
    }

    #[inline]
    pub fn decrease_expression_depth(&mut self) {
        self.expression_depth = self.expression_depth.saturating_sub(1);
    }
}

impl ParserControlContext {
    #[inline]
    pub fn get_sync_position(&self) -> Option<&ParserSyncPosition> {
        self.sync_position.last()
    }

    #[inline]
    pub fn get_expression_depth(&self) -> u32 {
        self.expression_depth
    }
}

#[derive(Debug, Default)]
pub struct ParserTypeContext {
    infered_types: Vec<Type>,
}

impl ParserTypeContext {
    #[inline]
    pub fn get_infered_type(&self) -> Option<Type> {
        self.infered_types.last().cloned()
    }
}

impl ParserTypeContext {
    #[inline]
    pub fn add_infered_type(&mut self, t: Type) {
        self.infered_types.push(t);
    }

    #[inline]
    pub fn pop_infered_type(&mut self) {
        self.infered_types.pop();
    }

    #[inline]
    pub fn reset_infered_types(&mut self) {
        self.infered_types.clear();
    }
}
