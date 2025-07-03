use crate::{
    core::errors::standard::ThrushCompilerIssue,
    frontend::lexer::{Lexer, span::Span},
};

#[inline]
pub fn check_float_format(lexer: &Lexer, lexeme: &str) -> Result<(), ThrushCompilerIssue> {
    let dot_count: usize = lexeme.bytes().filter(|&b| b == b'.').count();

    let span: Span = Span::new(lexer.line, lexer.span);

    if dot_count > 1 {
        return Err(ThrushCompilerIssue::Error(
            String::from("Syntax error"),
            String::from("Float number should only contain one dot."),
            None,
            span,
        ));
    }

    if lexeme.parse::<f32>().is_ok() {
        return Ok(());
    }

    if lexeme.parse::<f64>().is_ok() {
        return Ok(());
    }

    Err(ThrushCompilerIssue::Error(
        String::from("Syntax error"),
        String::from("Out of bounds float."),
        None,
        span,
    ))
}
