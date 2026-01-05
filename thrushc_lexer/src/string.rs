use thrushc_errors::{CompilationIssue, CompilationIssueCode};
use thrushc_span::Span;
use thrushc_token::{Token, tokentype::TokenType};

use crate::Lexer;

pub fn lex(lexer: &mut Lexer) -> Result<(), CompilationIssue> {
    lexer.start_span();

    let mut found_end_quote: bool = false;

    while !lexer.is_eof() {
        if lexer.peek() == '"' {
            lexer.advance_only();
            found_end_quote = true;

            break;
        }

        self::advance_string_char(lexer)?;
    }

    lexer.end_span();

    let span: Span = Span::new(lexer.line, lexer.span);

    let lexeme: String = lexer.shrink_lexeme();
    let bytes: Vec<u8> = lexer.shrink_lexeme_bytes();
    let ascii: String = self::convert_to_ascii(lexer, &lexeme);

    self::validate_and_finalize_string(lexer, found_end_quote, span, lexeme, ascii, bytes)?;

    Ok(())
}

fn handle_escape_sequence(lexer: &mut Lexer) -> Result<char, CompilationIssue> {
    lexer.advance_only();

    if lexer.is_eof() {
        lexer.end_span();

        let span: Span = Span::new(lexer.line, lexer.span);

        return Err(CompilationIssue::Error(
            CompilationIssueCode::E0001,
            "Unexpected EOF after escape character.".into(),
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

            Err(CompilationIssue::Error(
                CompilationIssueCode::E0001,
                "Invalid escape sequence. Valid escapes are '\\n', '\\t', '\\r', '\\0', '\\\\', '\\'', and '\\\"'.".into(),
                None,
                span,
            ))
        }
    }
}

fn advance_string_char(lexer: &mut Lexer) -> Result<char, CompilationIssue> {
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
    ascii: String,
    bytes: Vec<u8>,
) -> Result<(), CompilationIssue> {
    if !found_end_quote {
        return Err(CompilationIssue::Error(
            CompilationIssueCode::E0001,
            "Unclosed literal string. Did you forget to close it with a '\"'?".into(),
            None,
            span,
        ));
    }

    lexer.tokens.push(Token {
        lexeme,
        ascii,
        bytes,
        kind: TokenType::Str,
        span,
    });

    Ok(())
}

#[must_use]
pub fn convert_to_ascii(lexer: &Lexer, lexeme: &str) -> String {
    let mut scaped_unicode_string: String = String::with_capacity(lexeme.len());

    lexeme.chars().for_each(|char| {
        if lexer.is_ascii_char(char) {
            scaped_unicode_string.push(char);
        } else {
            let mut utf8_buf: [u8; 4] = [0u8; 4];

            let utf8_bytes: &[u8] = char.encode_utf8(&mut utf8_buf).as_bytes();

            utf8_bytes.iter().for_each(|byte| {
                scaped_unicode_string.push_str(&format!("{:02X}", byte));
            });
        }
    });

    scaped_unicode_string
}
