#[derive(Debug, Clone, Copy)]
pub struct CastMetadata {
    is_constant: bool,
    is_allocated: bool,
}

impl CastMetadata {
    pub fn new(is_constant: bool, is_allocated: bool) -> Self {
        Self {
            is_constant,
            is_allocated,
        }
    }

    #[inline]
    pub fn is_constant(&self) -> bool {
        self.is_constant
    }

    #[inline]
    pub fn is_allocated(&self) -> bool {
        self.is_allocated
    }
}
