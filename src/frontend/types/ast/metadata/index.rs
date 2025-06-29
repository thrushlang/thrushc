#[derive(Debug, Clone, Copy)]
pub struct IndexMetadata {
    is_mutable: bool,
}

impl IndexMetadata {
    pub fn new(is_mutable: bool) -> Self {
        Self { is_mutable }
    }

    pub fn is_mutable(&self) -> bool {
        self.is_mutable
    }
}
