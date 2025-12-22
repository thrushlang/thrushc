#[derive(Debug, Clone, Copy)]
pub struct TypeCheckerExprMetadata {
    is_literal: bool,
}

impl TypeCheckerExprMetadata {
    #[inline]
    pub fn new(is_literal: bool) -> Self {
        Self { is_literal }
    }
}

impl TypeCheckerExprMetadata {
    #[inline]
    pub fn is_literal(&self) -> bool {
        self.is_literal
    }
}
