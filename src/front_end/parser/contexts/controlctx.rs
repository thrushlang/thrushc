use crate::front_end::parser::contexts::sync::ParserSyncPosition;

#[derive(Debug)]
pub struct ParserControlContext {
    sync_position: Vec<ParserSyncPosition>,
}

impl ParserControlContext {
    #[inline]
    pub fn new() -> Self {
        Self {
            sync_position: Vec::with_capacity(100),
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
}

impl ParserControlContext {
    #[inline]
    pub fn get_sync_position(&self) -> Option<&ParserSyncPosition> {
        self.sync_position.last()
    }
}
