use super::super::frontend::lexer::Span;

#[derive(Debug, Clone)]
pub enum ThrushCompilerError {
    Error(String, String, Span),
}
