#[derive(Debug, Clone, Copy)]
pub struct CastMetadata {
    is_constant: bool,
}

impl CastMetadata {
    pub fn new(is_constant: bool) -> Self {
        Self { is_constant }
    }

    pub fn is_constant(&self) -> bool {
        self.is_constant
    }
}
