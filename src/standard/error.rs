use super::super::frontend::lexer::Span;

#[derive(Debug, Clone)]
pub enum ThrushCompilerIssue {
    Error(String, String, Option<String>, Span),
    Warning(String, String, Span),
}
