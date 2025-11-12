use crate::core::errors::standard::ThrushCompilerIssue;

use crate::front_end::lexer::Lexer;
use crate::front_end::lexer::span::Span;
use crate::front_end::lexer::token::Token;
use crate::front_end::lexer::tokentype::TokenType;

pub fn lex(lexer: &mut Lexer) -> Result<(), ThrushCompilerIssue> {
    let char: char = match lexer.advance() {
        '\\' => {
            lexer.end_span();
            let span: Span = Span::new(lexer.line, lexer.span);

            self::handle_char_scape_sequence(lexer, span)?
        }

        c => c,
    };

    lexer.end_span();

    let span: Span = Span::new(lexer.line, lexer.span);

    lexer.advance_only();

    if lexer.previous() != '\'' {
        return Err(ThrushCompilerIssue::Error(
            "Syntax error".into(),
            "Unclosed char. Did you forget to close the char with a '\''?.".into(),
            None,
            span,
        ));
    }

    lexer.tokens.push(Token {
        lexeme: char.to_string(),
        ascii: String::default(),
        bytes: Vec::default(),
        kind: TokenType::Char,
        span,
    });

    Ok(())
}

fn handle_char_scape_sequence(lexer: &mut Lexer, span: Span) -> Result<char, ThrushCompilerIssue> {
    match lexer.advance() {
        'n' => Ok('\n'),
        't' => Ok('\t'),
        'r' => Ok('\r'),
        '\\' => Ok('\\'),
        '0' => Ok('\0'),
        '\'' => Ok('\''),
        '"' => Ok('"'),

        _ => Err(ThrushCompilerIssue::Error(
            "Syntax error".into(),
            "Invalid escape sequence. Valid escapes are '\\n', '\\t', '\\r', '\\0', '\\\\', '\\'', and '\\\"'.".into(),
            None,
            span,
        )),
    }
}
