#[derive(Debug, Clone, Copy)]
pub struct ConstantMetadata {
    is_global: bool,
}

impl ConstantMetadata {
    pub fn new(is_global: bool) -> Self {
        Self { is_global }
    }

    pub fn is_global(&self) -> bool {
        self.is_global
    }
}
