use crate::{
    core::errors::standard::ThrushCompilerIssue,
    frontend::lexer::{Lexer, span::Span, token::Token, tokentype::TokenType},
};

pub fn lex(lexer: &mut Lexer) -> Result<(), ThrushCompilerIssue> {
    while lexer.is_char_boundary() {
        lexer.advance();
    }

    lexer.end_span();

    let span: Span = Span::new(lexer.line, lexer.span);

    if lexer.peek() != '\'' {
        return Err(ThrushCompilerIssue::Error(
            "Syntax error".into(),
            "Unclosed char. Did you forget to close the char with a '\''?.".into(),
            None,
            span,
        ));
    }

    lexer.advance();

    let lexeme: String = lexer.shrink_lexeme();

    if lexeme.len() > 1 {
        return Err(ThrushCompilerIssue::Error(
            "Syntax error".into(),
            "A character can only contain one byte.".into(),
            None,
            span,
        ));
    }

    lexer.tokens.push(Token {
        kind: TokenType::Char,
        ascii_lexeme: String::default(),
        lexeme,
        span,
    });

    Ok(())
}
