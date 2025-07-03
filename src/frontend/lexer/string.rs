use crate::{
    core::errors::standard::ThrushCompilerIssue,
    frontend::lexer::{Lexer, span::Span, token::Token, tokentype::TokenType},
};

pub fn lex(lexer: &mut Lexer) -> Result<(), ThrushCompilerIssue> {
    while lexer.is_string_boundary() {
        lexer.advance();
    }

    lexer.end_span();

    let span: Span = Span::new(lexer.line, lexer.span);

    if lexer.peek() != '"' {
        return Err(ThrushCompilerIssue::Error(
            String::from("Syntax error"),
            String::from("Unclosed literal string. Did you forget to close it with a '\"'?"),
            None,
            span,
        ));
    }

    lexer.advance();

    let lexeme: String = lexer.shrink_lexeme();

    lexer.tokens.push(Token {
        kind: TokenType::Str,
        ascii_lexeme: String::default(),
        lexeme,
        span,
    });

    Ok(())
}
