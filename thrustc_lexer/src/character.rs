use thrustc_errors::{CompilationIssue, CompilationIssueCode};
use thrustc_span::Span;
use thrustc_token::Token;
use thrustc_token_type::TokenType;

use crate::Lexer;

pub fn lex(lexer: &mut Lexer) -> Result<(), CompilationIssue> {
    let char: char = match lexer.advance() {
        '\\' => {
            lexer.end_span();
            let span: Span = Span::new(lexer.peek_span());

            self::handle_char_scape_sequence(lexer, span)?
        }

        c => c,
    };

    lexer.end_span();

    let span: Span = Span::new(lexer.peek_span());

    lexer.advance_only();

    println!("{}", lexer.current);

    if lexer.previous() != '\'' {
        return Err(CompilationIssue::Error(
            CompilationIssueCode::E0001,
            "Unclosed char. Did you forget to close the char with a '\''?.".into(),
            None,
            span,
        ));
    }

    lexer.tokens.push(Token {
        lexeme: char.to_string(),
        ascii: String::default(),
        kind: TokenType::Char,
        span,
    });

    Ok(())
}

fn handle_char_scape_sequence(lexer: &mut Lexer, span: Span) -> Result<char, CompilationIssue> {
    match lexer.advance() {
        'n' => Ok('\n'),
        't' => Ok('\t'),
        'r' => Ok('\r'),
        '\\' => Ok('\\'),
        '0' => Ok('\0'),
        '\'' => Ok('\''),
        '"' => Ok('"'),

        _ => Err(CompilationIssue::Error(
            CompilationIssueCode::E0001,
            "Invalid escape sequence. Valid escapes are '\\n', '\\t', '\\r', '\\0', '\\\\', '\\'', and '\\\"'.".into(),
            None,
            span,
        )),
    }
}
