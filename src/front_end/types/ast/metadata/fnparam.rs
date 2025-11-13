#[derive(Debug, Clone, Copy)]
pub struct FunctionParameterMetadata {
    is_mutable: bool,
}

impl FunctionParameterMetadata {
    #[inline]
    pub fn new(is_mutable: bool) -> Self {
        Self { is_mutable }
    }
}

impl FunctionParameterMetadata {
    #[inline]
    pub fn is_mutable(&self) -> bool {
        self.is_mutable
    }
}
