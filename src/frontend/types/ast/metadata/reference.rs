#[derive(Debug, Clone, Copy)]
pub struct ReferenceMetadata {
    is_allocated: bool,
    is_mutable: bool,
}

impl ReferenceMetadata {
    pub fn new(is_allocated: bool, is_mutable: bool) -> Self {
        Self {
            is_allocated,
            is_mutable,
        }
    }

    pub fn is_allocated(&self) -> bool {
        self.is_allocated
    }

    pub fn is_mutable(&self) -> bool {
        self.is_mutable
    }
}
