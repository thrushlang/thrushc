use crate::front_end::parser::contexts::sync::ParserSyncPosition;

#[derive(Debug)]
pub struct ParserControlContext {
    sync_position: Vec<ParserSyncPosition>,

    inside_function: bool,
}

impl ParserControlContext {
    #[inline]
    pub fn new() -> Self {
        Self {
            sync_position: Vec::with_capacity(100),
            inside_function: false,
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
    pub fn set_inside_function(&mut self, value: bool) {
        self.inside_function = value;
    }
}

impl ParserControlContext {
    #[inline]
    pub fn get_sync_position(&self) -> Option<&ParserSyncPosition> {
        self.sync_position.last()
    }

    #[inline]
    pub fn get_inside_function(&self) -> bool {
        self.inside_function
    }
}
