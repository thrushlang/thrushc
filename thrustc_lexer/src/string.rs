use thrustc_errors::{CompilationIssue, CompilationIssueCode};
use thrustc_span::Span;
use thrustc_token::Token;
use thrustc_token_type::TokenType;

use crate::Lexer;

pub fn lex(lexer: &mut Lexer, null_terminated: bool) -> Result<(), CompilationIssue> {
    lexer.start_span();

    let mut content: String = String::with_capacity(u8::MAX as usize);
    let mut found_end_quote: bool = false;

    while !lexer.is_eof() {
        if lexer.peek() == '"' {
            lexer.advance_only();
            found_end_quote = true;

            break;
        }

        let ch: char = self::advance_string_char(lexer)?;
        content.push(ch);
    }

    lexer.end_span();

    let span: Span = Span::new(lexer.line, lexer.span);

    if !null_terminated && content.chars().any(|ch| ch == '\0') {
        return Err(CompilationIssue::Error(
            CompilationIssueCode::E0001,
            "Invalid non null terminated string literal. The literal contains a null character."
                .into(),
            None,
            span,
        ));
    }

    let lexeme: String = content;
    let ascii: String = self::convert_to_ascii(lexer, &lexeme);

    self::validate_and_finalize_string(
        lexer,
        found_end_quote,
        null_terminated,
        span,
        lexeme,
        ascii,
    )?;

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
    null_terminated: bool,
    span: Span,
    lexeme: String,
    ascii: String,
) -> Result<(), CompilationIssue> {
    if !found_end_quote {
        return Err(CompilationIssue::Error(
            CompilationIssueCode::E0001,
            "Unclosed literal string. Did you forget to close it with a '\"'?".into(),
            None,
            span,
        ));
    }

    if null_terminated {
        lexer.tokens.push(Token {
            lexeme,
            ascii,
            kind: TokenType::CString,
            span,
        });
    } else {
        lexer.tokens.push(Token {
            lexeme,
            ascii,
            kind: TokenType::CNString,
            span,
        });
    }

    Ok(())
}

#[must_use]
pub fn convert_to_ascii(lexer: &Lexer, lexeme: &str) -> String {
    let mut scaped_unicode_string: String = String::with_capacity(lexeme.len());

    for char in lexeme.chars() {
        if lexer.is_ascii_char(char) {
            scaped_unicode_string.push(char);
            continue;
        }

        let mut utf8_buf: [u8; 4] = [0u8; 4];
        let utf8_bytes: &[u8] = char.encode_utf8(&mut utf8_buf).as_bytes();

        for byte in utf8_bytes {
            scaped_unicode_string.push_str(&format!("\\{:02X}", byte));
        }
    }

    scaped_unicode_string
}
