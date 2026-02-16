#[derive(Debug, Clone, Copy)]
pub struct TypeCheckerExpressionMetadata {
    is_literal: bool,
}

impl TypeCheckerExpressionMetadata {
    #[inline]
    pub fn new(is_literal: bool) -> Self {
        Self { is_literal }
    }
}

impl TypeCheckerExpressionMetadata {
    #[inline]
    pub fn is_literal(&self) -> bool {
        self.is_literal
    }
}
