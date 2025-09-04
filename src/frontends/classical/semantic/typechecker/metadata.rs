use crate::frontends::classical::{
    lexer::span::Span, semantic::typechecker::position::TypeCheckerPosition,
};

#[derive(Debug, Clone, Copy)]
pub struct TypeCheckerExprMetadata {
    is_literal: bool,
    position: Option<TypeCheckerPosition>,
    span: Span,
}

impl TypeCheckerExprMetadata {
    pub fn new(is_literal: bool, position: Option<TypeCheckerPosition>, span: Span) -> Self {
        Self {
            is_literal,
            position,
            span,
        }
    }

    pub fn is_literal(&self) -> bool {
        self.is_literal
    }

    pub fn get_position(&self) -> Option<TypeCheckerPosition> {
        self.position
    }

    pub fn get_span(&self) -> Span {
        self.span
    }
}
