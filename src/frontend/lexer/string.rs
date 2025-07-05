use crate::{
    core::errors::standard::ThrushCompilerIssue,
    frontend::lexer::{Lexer, span::Span, token::Token, tokentype::TokenType},
};

pub fn lex(lexer: &mut Lexer) -> Result<(), ThrushCompilerIssue> {
    let mut found_end_quote: bool = false;

    while !lexer.end() {
        if lexer.peek() == '"' {
            lexer.advance();
            found_end_quote = true;

            break;
        }

        self::advance_string_char(lexer)?;
    }

    let string_span: Span = Span::new(lexer.line, lexer.span);
    let raw_lexeme: String = lexer.shrink_lexeme();

    self::validate_and_finalize_string(lexer, found_end_quote, string_span, raw_lexeme)?;

    Ok(())
}

fn handle_escape_sequence(lexer: &mut Lexer) -> Result<char, ThrushCompilerIssue> {
    lexer.advance();

    if lexer.end() {
        lexer.end_span();

        let span: Span = Span::new(lexer.line, lexer.span);

        return Err(ThrushCompilerIssue::Error(
            "Syntax error".into(),
            "Unexpected end of file after escape character.".into(),
            None,
            span,
        ));
    }

    let escaped_char: char = lexer.advance();

    match escaped_char {
        'n' => Ok('\n'),
        't' => Ok('\t'),
        'r' => Ok('\r'),
        '\\' => Ok('\\'),
        '0' => Ok('\0'),
        '\'' => Ok('\''),
        '"' => Ok('"'),

        _ => {
            lexer.end_span();

            let span: Span = Span::new(lexer.line, lexer.span);

            Err(ThrushCompilerIssue::Error(
                "Syntax error".into(),
                "Invalid escape sequence. Valid escapes are '\\n', '\\t', '\\r', '\\0', '\\\\', '\\'', and '\\\"'.".into(),
                None,
                span,
            ))
        }
    }
}

fn advance_string_char(lexer: &mut Lexer) -> Result<char, ThrushCompilerIssue> {
    let current_char: char = lexer.peek();

    if current_char == '\\' {
        self::handle_escape_sequence(lexer)
    } else {
        Ok(lexer.advance())
    }
}

fn validate_and_finalize_string(
    lexer: &mut Lexer,
    found_end_quote: bool,
    span: Span,
    lexeme: String,
) -> Result<(), ThrushCompilerIssue> {
    if !found_end_quote {
        return Err(ThrushCompilerIssue::Error(
            "Syntax error".into(),
            "Unclosed literal string. Did you forget to close it with a '\"'?".into(),
            None,
            span,
        ));
    }

    lexer.tokens.push(Token {
        kind: TokenType::Str,
        ascii_lexeme: String::default(),
        lexeme,
        span,
    });

    Ok(())
}
