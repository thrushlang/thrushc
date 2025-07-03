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
            String::from("Syntax error"),
            String::from("Unclosed char. Did you forget to close the char with a \'?"),
            None,
            span,
        ));
    }

    lexer.advance();

    if lexer.shrink_lexeme().len() > 1 {
        return Err(ThrushCompilerIssue::Error(
            String::from("Syntax error"),
            String::from("A char data type only can contain one character."),
            None,
            span,
        ));
    }

    let lexeme: String = lexer.shrink_lexeme();

    lexer.tokens.push(Token {
        kind: TokenType::Char,
        ascii_lexeme: String::default(),
        lexeme,
        span,
    });

    Ok(())
}
