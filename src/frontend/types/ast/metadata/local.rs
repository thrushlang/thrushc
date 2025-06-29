#[derive(Debug, Clone, Copy)]
pub struct LocalMetadata {
    is_undefined: bool,
    is_mutable: bool,
}

impl LocalMetadata {
    pub fn new(is_undefined: bool, is_mutable: bool) -> Self {
        Self {
            is_undefined,
            is_mutable,
        }
    }

    pub fn is_undefined(&self) -> bool {
        self.is_undefined
    }

    pub fn is_mutable(&self) -> bool {
        self.is_mutable
    }
}
