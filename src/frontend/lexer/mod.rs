use std::{mem, process};

use keywords::THRUSH_KEYWORDS;
use span::Span;
use token::Token;

use crate::middle::types::frontend::lexer::tokenkind::TokenKind;

use crate::standard::errors::lexer::ThrushLexerPanic;
use crate::standard::logging::LoggingType;
use crate::standard::misc::CompilerFile;

use super::super::standard::{
    constants::MINIMAL_ERROR_CAPACITY, diagnostic::Diagnostician, error::ThrushCompilerIssue,
};

pub mod keywords;
pub mod span;
pub mod token;

const MINIMAL_TOKENS_CAPACITY: usize = 1_000_000;
const MAXIMUM_BYTES_TO_LEX: usize = 1_000_000;

pub struct Lexer<'a> {
    tokens: Vec<Token<'a>>,
    errors: Vec<ThrushCompilerIssue>,
    code: &'a [u8],
    start: usize,
    current: usize,
    line: usize,
    span: (usize, usize),
    diagnostician: Diagnostician,
}

impl<'a> Lexer<'a> {
    pub fn lex(code: &'a [u8], file: &'a CompilerFile) -> Result<Vec<Token<'a>>, ThrushLexerPanic> {
        Self {
            tokens: Vec::with_capacity(MINIMAL_TOKENS_CAPACITY),
            errors: Vec::with_capacity(MINIMAL_ERROR_CAPACITY),
            code,
            start: 0,
            current: 0,
            line: 1,
            span: (0, 0),
            diagnostician: Diagnostician::new(file),
        }
        .start()
    }

    fn start(&mut self) -> Result<Vec<Token<'a>>, ThrushLexerPanic> {
        if self.code.len() > MAXIMUM_BYTES_TO_LEX {
            return Err(ThrushLexerPanic::TooBigFile(
                self.diagnostician.get_file_path(),
            ));
        }

        while !self.end() {
            self.start = self.current;
            self.start_span();

            if let Err(error) = self.scan() {
                self.errors.push(error)
            }
        }

        if !self.errors.is_empty() {
            self.errors.iter().for_each(|error| {
                self.diagnostician
                    .build_diagnostic(error, LoggingType::Error);
            });

            process::exit(1);
        };

        self.tokens.push(Token {
            lexeme: "",
            kind: TokenKind::Eof,
            span: Span::new(self.line, self.span),
        });

        Ok(mem::take(&mut self.tokens))
    }

    fn scan(&mut self) -> Result<(), ThrushCompilerIssue> {
        match self.advance() {
            b'[' => self.make(TokenKind::LBracket),
            b']' => self.make(TokenKind::RBracket),
            b'(' => self.make(TokenKind::LParen),
            b')' => self.make(TokenKind::RParen),
            b'{' => self.make(TokenKind::LBrace),
            b'}' => self.make(TokenKind::RBrace),
            b',' => self.make(TokenKind::Comma),
            b'.' if self.char_match(b'.') && self.char_match(b'.') => self.make(TokenKind::Pass),
            b'.' if self.char_match(b'.') => self.make(TokenKind::Range),
            b'.' => self.make(TokenKind::Dot),
            b'%' => self.make(TokenKind::Arith),
            b'*' => self.make(TokenKind::Star),
            b'/' if self.char_match(b'/') => loop {
                if self.peek() == b'\n' || self.end() {
                    break;
                }

                self.advance();
            },
            b'/' if self.char_match(b'*') => loop {
                if self.char_match(b'*') && self.char_match(b'/') {
                    break;
                } else if self.end() {
                    self.end_span();

                    let span: Span = Span::new(self.line, self.span);

                    return Err(ThrushCompilerIssue::Error(
                        String::from("Syntax Error"),
                        String::from(
                            "Unterminated multiline comment. Did you forget to close the comment with a '*/'?",
                        ),
                        None,
                        span,
                    ));
                }

                self.advance();
            },
            b'/' => self.make(TokenKind::Slash),
            b';' => self.make(TokenKind::SemiColon),
            b'-' if self.char_match(b'-') => self.make(TokenKind::MinusMinus),
            b'-' if self.char_match(b'=') => self.make(TokenKind::MinusEq),
            b'-' if self.char_match(b'>') => self.make(TokenKind::Arrow),
            b'-' => self.make(TokenKind::Minus),
            b'+' if self.char_match(b'+') => self.make(TokenKind::PlusPlus),
            b'+' if self.char_match(b'=') => self.make(TokenKind::PlusEq),
            b'+' => self.make(TokenKind::Plus),
            b':' if self.char_match(b':') => self.make(TokenKind::ColonColon),
            b':' => self.make(TokenKind::Colon),
            b'!' if self.char_match(b'=') => self.make(TokenKind::BangEq),
            b'!' => self.make(TokenKind::Bang),
            b'=' if self.char_match(b'=') => self.make(TokenKind::EqEq),
            b'=' => self.make(TokenKind::Eq),
            b'<' if self.char_match(b'=') => self.make(TokenKind::LessEq),
            b'<' if self.char_match(b'<') => self.make(TokenKind::LShift),
            b'<' => self.make(TokenKind::Less),
            b'>' if self.char_match(b'=') => self.make(TokenKind::GreaterEq),
            b'>' if self.char_match(b'>') => self.make(TokenKind::RShift),
            b'>' => self.make(TokenKind::Greater),
            b'|' if self.char_match(b'|') => self.make(TokenKind::Or),
            b'&' if self.char_match(b'&') => self.make(TokenKind::And),
            b' ' | b'\r' | b'\t' => {}
            b'\n' => self.line += 1,
            b'\'' => self.char()?,
            b'"' => self.string()?,
            b'0'..=b'9' => self.number()?,
            b'a'..=b'z' | b'A'..=b'Z' | b'_' | b'@' => self.identifier()?,
            _ => {
                self.end_span();

                let span: Span = Span::new(self.line, self.span);

                return Err(ThrushCompilerIssue::Error(
                    String::from("Unknown character"),
                    String::from("The compiler does not know how to handle this character."),
                    None,
                    span,
                ));
            }
        }

        Ok(())
    }

    fn identifier(&mut self) -> Result<(), ThrushCompilerIssue> {
        while self.is_alpha(self.peek())
            || self.peek().is_ascii_digit()
            || self.peek() == b'!' && self.peek() != b':'
        {
            self.advance();
        }

        let code: &[u8] = &self.code[self.start..self.current];

        if let Some(token_type) = THRUSH_KEYWORDS.get(code) {
            self.make(*token_type);
        } else {
            self.make(TokenKind::Identifier);
        }

        Ok(())
    }

    fn number(&mut self) -> Result<(), ThrushCompilerIssue> {
        let mut is_hexadecimal: bool = false;
        let mut is_binary: bool = false;

        while self.peek().is_ascii_digit()
            || self.peek() == b'_'
            || self.peek() == b'.'
            || self.peek() == b'x'
            || self.peek() == b'b'
            || is_hexadecimal
            || is_binary
        {
            if is_hexadecimal && self.previous() == b'0' && self.peek() == b'x' {
                self.end_span();

                return Err(ThrushCompilerIssue::Error(
                    String::from("Syntax error"),
                    String::from("Hexadecimal identifier '0x' cannot be repeated."),
                    None,
                    Span::new(self.line, self.span),
                ));
            }

            if is_binary && self.previous() == b'0' && self.peek() == b'b' {
                self.end_span();

                return Err(ThrushCompilerIssue::Error(
                    String::from("Syntax error"),
                    String::from("Binary identifier '0b' cannot be repeated."),
                    None,
                    Span::new(self.line, self.span),
                ));
            }

            if is_hexadecimal && !self.peek().is_ascii_alphanumeric() {
                self.end_span();
                break;
            }

            if is_binary && !self.peek().is_ascii_digit() {
                self.end_span();
                break;
            }

            if self.peek() == b'x' && self.peek_next().is_ascii_alphanumeric() {
                is_hexadecimal = true;
            }

            if self.peek() == b'b' && self.peek_next().is_ascii_digit() {
                is_binary = true;
            }

            self.advance();
        }

        self.end_span();

        let lexeme: &str = self.lexeme();

        self.check_number(lexeme)?;

        let span: Span = Span::new(self.line, self.span);

        if lexeme.contains(".") {
            self.tokens.push(Token {
                lexeme,
                kind: TokenKind::Float,
                span,
            });

            return Ok(());
        }

        self.tokens.push(Token {
            lexeme,
            kind: TokenKind::Integer,
            span,
        });

        Ok(())
    }

    #[inline]
    fn check_number(&mut self, lexeme: &str) -> Result<(), ThrushCompilerIssue> {
        if lexeme.contains('.') {
            return self.parse_float(lexeme);
        }

        self.parse_integer(lexeme)
    }

    #[inline]
    fn parse_float(&self, lexeme: &str) -> Result<(), ThrushCompilerIssue> {
        let dot_count: usize = lexeme.bytes().filter(|&b| b == b'.').count();

        let span: Span = Span::new(self.line, self.span);

        if dot_count > 1 {
            return Err(ThrushCompilerIssue::Error(
                String::from("Syntax error"),
                String::from("Float values should only contain one dot."),
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
            String::from("Out of bounds."),
            None,
            span,
        ))
    }

    #[inline]
    fn parse_integer(&self, lexeme: &str) -> Result<(), ThrushCompilerIssue> {
        const I8_MIN: isize = -128;
        const I8_MAX: isize = 127;
        const I16_MIN: isize = -32768;
        const I16_MAX: isize = 32767;
        const I32_MIN: isize = -2147483648;
        const I32_MAX: isize = 2147483647;

        const U8_MIN: usize = 0;
        const U8_MAX: usize = 255;
        const U16_MIN: usize = 0;
        const U16_MAX: usize = 65535;
        const U32_MIN: usize = 0;
        const U32_MAX: usize = 4294967295;

        let span: Span = Span::new(self.line, self.span);

        if lexeme.starts_with("0x") {
            let cleaned_lexeme: String = lexeme
                .strip_prefix("0x")
                .unwrap_or(&lexeme.replace("0x", ""))
                .replace("_", "");

            return match isize::from_str_radix(&cleaned_lexeme, 16) {
                Ok(num) => {
                    if (I8_MIN..=I8_MAX).contains(&num)
                        || (I16_MIN..=I16_MAX).contains(&num)
                        || (I32_MIN..=I32_MAX).contains(&num)
                        || (isize::MIN..=isize::MAX).contains(&num)
                    {
                        return Ok(());
                    } else {
                        return Err(ThrushCompilerIssue::Error(
                            String::from("Syntax error"),
                            String::from("Out of bounds signed hexadecimal format."),
                            None,
                            span,
                        ));
                    }
                }

                Err(_) => match usize::from_str_radix(&cleaned_lexeme, 16) {
                    Ok(num) => {
                        if (U8_MIN..=U8_MAX).contains(&num)
                            || (U16_MIN..=U16_MAX).contains(&num)
                            || (U32_MIN..=U32_MAX).contains(&num)
                            || (usize::MIN..=usize::MAX).contains(&num)
                        {
                            return Ok(());
                        } else {
                            return Err(ThrushCompilerIssue::Error(
                                String::from("Syntax error"),
                                String::from("Out of bounds unsigned hexadecimal format."),
                                None,
                                span,
                            ));
                        }
                    }

                    Err(_) => Err(ThrushCompilerIssue::Error(
                        String::from("Syntax error"),
                        String::from("Invalid numeric hexadecimal format."),
                        None,
                        span,
                    )),
                },
            };
        }

        if lexeme.starts_with("0b") {
            let cleaned_lexeme: String = lexeme
                .strip_prefix("0b")
                .unwrap_or(&lexeme.replace("0b", ""))
                .replace("_", "");

            return match isize::from_str_radix(&cleaned_lexeme, 2) {
                Ok(num) => {
                    if (I8_MIN..=I8_MAX).contains(&num)
                        || (I16_MIN..=I16_MAX).contains(&num)
                        || (I32_MIN..=I32_MAX).contains(&num)
                        || (isize::MIN..=isize::MAX).contains(&num)
                    {
                        return Ok(());
                    } else {
                        return Err(ThrushCompilerIssue::Error(
                            String::from("Syntax error"),
                            String::from("Out of bounds signed binary format."),
                            None,
                            span,
                        ));
                    }
                }

                Err(_) => match usize::from_str_radix(&cleaned_lexeme, 2) {
                    Ok(num) => {
                        if (U8_MIN..=U8_MAX).contains(&num)
                            || (U16_MIN..=U16_MAX).contains(&num)
                            || (U32_MIN..=U32_MAX).contains(&num)
                            || (usize::MIN..=usize::MAX).contains(&num)
                        {
                            return Ok(());
                        } else {
                            return Err(ThrushCompilerIssue::Error(
                                String::from("Syntax error"),
                                String::from("Out of bounds unsigned binary format."),
                                None,
                                span,
                            ));
                        }
                    }

                    Err(_) => Err(ThrushCompilerIssue::Error(
                        String::from("Syntax error"),
                        String::from("Invalid binary format."),
                        None,
                        span,
                    )),
                },
            };
        }

        match lexeme.parse::<usize>() {
            Ok(num) => {
                if (U8_MIN..=U8_MAX).contains(&num)
                    || (U16_MIN..=U16_MAX).contains(&num)
                    || (U32_MIN..=U32_MAX).contains(&num)
                    || (usize::MIN..=usize::MAX).contains(&num)
                {
                    Ok(())
                } else {
                    Err(ThrushCompilerIssue::Error(
                        String::from("Syntax error"),
                        String::from("Out of bounds."),
                        None,
                        span,
                    ))
                }
            }

            Err(_) => match lexeme.parse::<isize>() {
                Ok(num) => {
                    if (I8_MIN..=I8_MAX).contains(&num)
                        || (I16_MIN..=I16_MAX).contains(&num)
                        || (I32_MIN..=I32_MAX).contains(&num)
                        || (isize::MIN..=isize::MAX).contains(&num)
                    {
                        Ok(())
                    } else {
                        Err(ThrushCompilerIssue::Error(
                            String::from("Syntax error"),
                            String::from("Out of bounds."),
                            None,
                            span,
                        ))
                    }
                }

                Err(_) => Err(ThrushCompilerIssue::Error(
                    String::from("Syntax error"),
                    String::from("Out of bounds."),
                    None,
                    span,
                )),
            },
        }
    }

    fn char(&mut self) -> Result<(), ThrushCompilerIssue> {
        while self.peek() != b'\'' && !self.end() {
            self.advance();
        }

        self.end_span();

        let span: Span = Span::new(self.line, self.span);

        if self.peek() != b'\'' {
            return Err(ThrushCompilerIssue::Error(
                String::from("Syntax error"),
                String::from("Unclosed char. Did you forget to close the char with a \'?"),
                None,
                span,
            ));
        }

        self.advance();

        if self.code[self.start + 1..self.current - 1].len() > 1 {
            return Err(ThrushCompilerIssue::Error(
                String::from("Syntax error"),
                String::from("A char data type only can contain one character."),
                None,
                span,
            ));
        }

        let lexeme: &str = self.shrink_lexeme();

        self.tokens.push(Token {
            kind: TokenKind::Char,
            lexeme,
            span,
        });

        Ok(())
    }

    fn string(&mut self) -> Result<(), ThrushCompilerIssue> {
        while self.is_string_boundary() {
            self.advance();
        }

        self.end_span();

        let span: Span = Span::new(self.line, self.span);

        if self.peek() != b'"' {
            return Err(ThrushCompilerIssue::Error(
                String::from("Syntax error"),
                String::from(
                    "Unclosed literal str. Did you forget to close the literal str with a '\"'?",
                ),
                None,
                span,
            ));
        }

        self.advance();

        let lexeme: &str = self.shrink_lexeme();

        self.tokens.push(Token {
            kind: TokenKind::Str,
            lexeme,
            span,
        });

        Ok(())
    }

    fn make(&mut self, kind: TokenKind) {
        self.end_span();

        self.tokens.push(Token {
            lexeme: self.lexeme(),
            kind,
            span: Span::new(self.line, self.span),
        });
    }

    fn char_match(&mut self, ch: u8) -> bool {
        if !self.end() && self.code[self.current] == ch {
            self.current += 1;
            return true;
        }

        false
    }

    fn advance(&mut self) -> u8 {
        let char: u8 = self.code[self.current];
        self.current += 1;

        char
    }

    #[must_use]
    #[inline]
    fn lexeme(&self) -> &'a str {
        core::str::from_utf8(&self.code[self.start..self.current]).unwrap_or("�")
    }

    #[must_use]
    #[inline]
    fn shrink_lexeme(&self) -> &'a str {
        core::str::from_utf8(&self.code[self.start + 1..self.current - 1]).unwrap_or("�")
    }

    #[inline]
    fn start_span(&mut self) {
        self.span.0 = self.start;
    }

    #[inline]
    fn end_span(&mut self) {
        self.span.1 = self.current;
    }

    #[inline]
    fn peek_next(&self) -> u8 {
        if self.current + 1 >= self.code.len() {
            return b'\0';
        }

        self.code[self.current + 1]
    }

    #[must_use]
    fn previous(&self) -> u8 {
        self.code[self.current - 1]
    }

    #[must_use]
    #[inline]
    fn peek(&self) -> u8 {
        if self.end() {
            return b'\0';
        }

        self.code[self.current]
    }

    #[must_use]
    #[inline]
    fn is_string_boundary(&self) -> bool {
        self.peek() != b'"' && !self.end()
    }

    #[must_use]
    #[inline]
    fn end(&self) -> bool {
        self.current >= self.code.len()
    }

    #[must_use]
    #[inline]
    fn is_alpha(&self, char: u8) -> bool {
        char.is_ascii_lowercase() || char.is_ascii_uppercase() || char == b'_'
    }
}
