use crate::frontend::lexer::span::Span;

use super::position::CompilationPosition;

#[derive(Debug, Clone)]
pub enum ThrushCompilerIssue {
    Error(String, String, Option<String>, Span),
    Warning(String, String, Span),
    FrontEndBug(String, String, Span, CompilationPosition, u32),
}

impl ThrushCompilerIssue {
    #[inline]
    pub fn is_bug(&self) -> bool {
        matches!(self, ThrushCompilerIssue::FrontEndBug(..))
    }
}
