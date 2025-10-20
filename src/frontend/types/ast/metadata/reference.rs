#[derive(Debug, Clone, Copy)]
pub struct ReferenceMetadata {
    is_allocated: bool,
    is_mutable: bool,
    reference_type: ReferenceType,
}

#[derive(Debug, Clone, Copy, Default)]
pub enum ReferenceType {
    Constant,

    #[default]
    None,
}

impl ReferenceMetadata {
    pub fn new(is_allocated: bool, is_mutable: bool, reference_type: ReferenceType) -> Self {
        Self {
            is_allocated,
            is_mutable,
            reference_type,
        }
    }

    #[inline]
    pub fn is_allocated(&self) -> bool {
        self.is_allocated
    }

    #[inline]
    pub fn is_mutable(&self) -> bool {
        self.is_mutable
    }
}

impl ReferenceMetadata {
    pub fn is_constant(&self) -> bool {
        matches!(self.reference_type, ReferenceType::Constant)
    }
}
