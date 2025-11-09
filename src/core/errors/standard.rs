use std::path::PathBuf;

use crate::front_end::lexer::span::Span;

use super::position::CompilationPosition;

#[derive(Debug, Clone)]
pub enum ThrushCompilerIssue {
    Error(String, String, Option<String>, Span),
    Warning(String, String, Span),
    FrontEndBug(String, String, Span, CompilationPosition, PathBuf, u32),
    BackenEndBug(String, String, Span, CompilationPosition, PathBuf, u32),
}

impl ThrushCompilerIssue {
    #[inline]
    pub fn is_bug(&self) -> bool {
        matches!(
            self,
            ThrushCompilerIssue::FrontEndBug(..) | ThrushCompilerIssue::BackenEndBug(..)
        )
    }
}
