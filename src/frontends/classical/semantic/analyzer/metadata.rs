use crate::frontends::classical::{
    lexer::span::Span, semantic::analyzer::position::TypeCheckerPosition,
};

#[derive(Debug, Clone, Copy)]
pub struct TypeCheckerExprMetadata {
    is_literal: bool,
    position: Option<TypeCheckerPosition>,
    span: Span,
}

impl TypeCheckerExprMetadata {
    #[inline]
    pub fn new(is_literal: bool, position: Option<TypeCheckerPosition>, span: Span) -> Self {
        Self {
            is_literal,
            position,
            span,
        }
    }
}

impl TypeCheckerExprMetadata {
    #[inline]
    pub fn is_literal(&self) -> bool {
        self.is_literal
    }

    #[inline]
    pub fn get_position(&self) -> Option<TypeCheckerPosition> {
        self.position
    }

    #[inline]
    pub fn get_span(&self) -> Span {
        self.span
    }
}
