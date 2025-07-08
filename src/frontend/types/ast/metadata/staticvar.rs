#[derive(Debug, Clone, Copy)]
pub struct StaticMetadata {
    is_global: bool,
    is_mutable: bool,
}

impl StaticMetadata {
    pub fn new(is_global: bool, is_mutable: bool) -> Self {
        Self {
            is_global,
            is_mutable,
        }
    }

    pub fn is_mutable(&self) -> bool {
        self.is_mutable
    }

    pub fn is_global(&self) -> bool {
        self.is_global
    }
}
