use std::path::PathBuf;

use crate::front_end::lexer::span::Span;

use super::position::CompilationPosition;

#[derive(Debug, Clone)]
pub enum CompilationIssue {
    Error(String, String, Option<String>, Span),
    Warning(String, String, Span),

    FrontEndBug(String, String, Span, CompilationPosition, PathBuf, u32),
    BackenEndBug(String, String, Span, CompilationPosition, PathBuf, u32),
}

impl CompilationIssue {
    #[inline]
    pub fn is_bug(&self) -> bool {
        matches!(
            self,
            CompilationIssue::FrontEndBug(..) | CompilationIssue::BackenEndBug(..)
        )
    }
}
