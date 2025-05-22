use crate::frontend::lexer::span::Span;

use super::position::CompilationPosition;

#[derive(Debug, Clone)]
pub enum ThrushCompilerIssue {
    Error(String, String, Option<String>, Span),
    Warning(String, String, Span),
    Bug(String, String, Span, CompilationPosition, u32),
}
impl ThrushCompilerIssue {
    #[inline]
    pub fn is_common_error(&self) -> bool {
        matches!(self, ThrushCompilerIssue::Error(..))
    }

    #[inline]
    pub fn is_common_warning(&self) -> bool {
        matches!(self, ThrushCompilerIssue::Warning(..))
    }

    #[inline]
    pub fn is_compiler_bug(&self) -> bool {
        matches!(self, ThrushCompilerIssue::Bug(..))
    }
}
