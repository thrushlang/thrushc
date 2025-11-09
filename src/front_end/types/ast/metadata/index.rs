#[derive(Debug, Clone, Copy)]
pub struct IndexMetadata {
    is_mutable: bool,
}

impl IndexMetadata {
    #[inline]
    pub fn new(is_mutable: bool) -> Self {
        Self { is_mutable }
    }
}

impl IndexMetadata {
    #[inline]
    pub fn is_mutable(&self) -> bool {
        self.is_mutable
    }
}
