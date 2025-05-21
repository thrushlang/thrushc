use crate::frontend::lexer::span::Span;

pub enum ThrushCompilerIssue {
    Error(String, String, Option<String>, Span),
    Warning(String, String, Span),
}
