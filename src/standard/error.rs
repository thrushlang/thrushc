use crate::frontend::lexer::span::Span;

#[derive(Debug, Clone)]
pub enum ThrushCompilerIssue {
    Error(String, String, Option<String>, Span),
    Warning(String, String, Span),
}
