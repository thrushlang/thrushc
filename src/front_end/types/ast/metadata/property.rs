#[derive(Debug, Clone, Copy)]
pub struct PropertyMetadata {
    is_allocated: bool,
}

impl PropertyMetadata {
    #[inline]
    pub fn new(is_allocated: bool) -> Self {
        Self { is_allocated }
    }
}

impl PropertyMetadata {
    #[inline]
    pub fn is_allocated(&self) -> bool {
        self.is_allocated
    }
}
