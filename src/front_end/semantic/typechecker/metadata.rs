use crate::core::diagnostic::span::Span;

#[derive(Debug, Clone, Copy)]
pub struct TypeCheckerExprMetadata {
    is_literal: bool,
    span: Span,
}

impl TypeCheckerExprMetadata {
    #[inline]
    pub fn new(is_literal: bool, span: Span) -> Self {
        Self { is_literal, span }
    }
}

impl TypeCheckerExprMetadata {
    #[inline]
    pub fn is_literal(&self) -> bool {
        self.is_literal
    }

    #[inline]
    pub fn get_span(&self) -> Span {
        self.span
    }
}
