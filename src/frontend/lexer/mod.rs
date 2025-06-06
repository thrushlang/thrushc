use std::{mem, process};

use keywords::THRUSH_KEYWORDS;
use span::Span;
use token::Token;

use crate::{
    core::{
        compiler::options::CompilerFile,
        console::logging::LoggingType,
        diagnostic::diagnostician::Diagnostician,
        errors::{lexer::ThrushLexerPanic, standard::ThrushCompilerIssue},
    },
    frontend::lexer::tokenkind::TokenKind,
};

pub mod keywords;
pub mod span;
pub mod token;
pub mod tokenkind;

const MAXIMUM_TOKENS_CAPACITY: usize = 1_000_000;
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
            tokens: Vec::with_capacity(MAXIMUM_TOKENS_CAPACITY),
            errors: Vec::with_capacity(100),
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
            if self.tokens.len() >= MAXIMUM_TOKENS_CAPACITY {
                return Err(ThrushLexerPanic::TooMuchTokens);
            }

            self.start = self.current;
            self.start_span();

            if let Err(error) = self.analyze() {
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

    fn analyze(&mut self) -> Result<(), ThrushCompilerIssue> {
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
                        "Syntax error".into(),
                        "Unterminated multiline comment. Did you forget to close the comment with a '*/'?".into(),
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
                    "Unknown character".into(),
                    "This character isn't recognized.".into(),
                    None,
                    span,
                ));
            }
        }

        Ok(())
    }

    fn identifier(&mut self) -> Result<(), ThrushCompilerIssue> {
        while self.is_identifier_boundary() {
            self.advance();
        }

        let lexeme: &[u8] = &self.code[self.start..self.current];

        if let Some(keyword) = THRUSH_KEYWORDS.get(lexeme) {
            self.make(*keyword);
        } else {
            self.make(TokenKind::Identifier);
        }

        Ok(())
    }

    fn number(&mut self) -> Result<(), ThrushCompilerIssue> {
        let mut is_hexadecimal: bool = false;
        let mut is_binary: bool = false;

        while self.is_number_boundary(is_hexadecimal, is_binary) {
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

        let span: Span = Span::new(self.line, self.span);

        let lexeme: &str = self.lexeme(span)?;

        if lexeme.contains(".") {
            self.check_float_format(lexeme)?;

            self.tokens.push(Token {
                lexeme,
                kind: TokenKind::Float,
                span,
            });

            return Ok(());
        }

        self.check_integer_format(lexeme)?;

        self.tokens.push(Token {
            lexeme,
            kind: TokenKind::Integer,
            span,
        });

        Ok(())
    }

    #[inline]
    fn check_float_format(&self, lexeme: &str) -> Result<(), ThrushCompilerIssue> {
        let dot_count: usize = lexeme.bytes().filter(|&b| b == b'.').count();

        let span: Span = Span::new(self.line, self.span);

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

    #[inline]
    fn check_integer_format(&self, lexeme: &str) -> Result<(), ThrushCompilerIssue> {
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
            return self.check_integer_hex_format(lexeme, span);
        }

        if lexeme.starts_with("0b") {
            return self.check_integer_binary_format(lexeme, span);
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
                        String::from("Out of bounds integer."),
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
                            String::from("Out of bounds integer."),
                            None,
                            span,
                        ))
                    }
                }

                Err(_) => Err(ThrushCompilerIssue::Error(
                    String::from("Syntax error"),
                    String::from("Out of bounds integer."),
                    None,
                    span,
                )),
            },
        }
    }

    fn check_integer_binary_format(
        &self,
        lexeme: &str,
        span: Span,
    ) -> Result<(), ThrushCompilerIssue> {
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

        let cleaned_lexeme: String = lexeme
            .strip_prefix("0b")
            .unwrap_or(&lexeme.replace("0b", ""))
            .replace("_", "");

        match isize::from_str_radix(&cleaned_lexeme, 2) {
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
                        String::from("Out of bounds signed binary format."),
                        None,
                        span,
                    ))
                }
            }

            Err(_) => match usize::from_str_radix(&cleaned_lexeme, 2) {
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
                            String::from("Out of bounds unsigned binary format."),
                            None,
                            span,
                        ))
                    }
                }

                Err(_) => Err(ThrushCompilerIssue::Error(
                    String::from("Syntax error"),
                    String::from("Invalid binary format."),
                    None,
                    span,
                )),
            },
        }
    }

    fn check_integer_hex_format(
        &self,
        lexeme: &str,
        span: Span,
    ) -> Result<(), ThrushCompilerIssue> {
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

        let cleaned_lexeme: String = lexeme
            .strip_prefix("0x")
            .unwrap_or(&lexeme.replace("0x", ""))
            .replace("_", "");

        match isize::from_str_radix(&cleaned_lexeme, 16) {
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
                        String::from("Out of bounds signed hexadecimal format."),
                        None,
                        span,
                    ))
                }
            }

            Err(_) => match usize::from_str_radix(&cleaned_lexeme, 16) {
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
                            String::from("Out of bounds unsigned hexadecimal format."),
                            None,
                            span,
                        ))
                    }
                }

                Err(_) => Err(ThrushCompilerIssue::Error(
                    String::from("Syntax error"),
                    String::from("Invalid numeric hexadecimal format."),
                    None,
                    span,
                )),
            },
        }
    }

    fn char(&mut self) -> Result<(), ThrushCompilerIssue> {
        while self.is_char_boundary() {
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

        let lexeme: &str = self.shrink_lexeme(span)?;

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
                String::from("Unclosed literal string. Did you forget to close it with a '\"'?"),
                None,
                span,
            ));
        }

        self.advance();

        let lexeme: &str = self.shrink_lexeme(span)?;

        self.tokens.push(Token {
            kind: TokenKind::Str,
            lexeme,
            span,
        });

        Ok(())
    }

    fn make(&mut self, kind: TokenKind) {
        self.end_span();

        let span: Span = Span::new(self.line, self.span);

        let lexeme: Result<&str, ThrushCompilerIssue> = self.lexeme(span);

        match lexeme {
            Ok(lexeme) => {
                self.tokens.push(Token { lexeme, kind, span });
            }
            Err(error) => {
                self.add_error(error);
            }
        }
    }

    fn char_match(&mut self, byte: u8) -> bool {
        if !self.end() && self.code[self.current] == byte {
            self.current += 1;
            return true;
        }

        false
    }

    fn advance(&mut self) -> u8 {
        let byte: u8 = self.code[self.current];
        self.current += 1;

        byte
    }

    #[inline]
    fn lexeme(&self, span: Span) -> Result<&'a str, ThrushCompilerIssue> {
        if let Ok(lexeme) = core::str::from_utf8(&self.code[self.start..self.current]) {
            return Ok(lexeme);
        }

        Err(ThrushCompilerIssue::Error(
            "Syntax error".into(),
            "Invalid utf-8 code.".into(),
            None,
            span,
        ))
    }

    #[inline]
    fn shrink_lexeme(&self, span: Span) -> Result<&'a str, ThrushCompilerIssue> {
        if let Ok(lexeme) = core::str::from_utf8(&self.code[self.start + 1..self.current - 1]) {
            return Ok(lexeme);
        }

        Err(ThrushCompilerIssue::Error(
            "Syntax error".into(),
            "Invalid utf-8 code.".into(),
            None,
            span,
        ))
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
    fn is_number_boundary(&self, is_hexadecimal: bool, is_binary: bool) -> bool {
        self.peek().is_ascii_digit()
            || self.peek() == b'_'
            || self.peek() == b'.'
            || self.peek() == b'x'
            || self.peek() == b'b'
            || is_hexadecimal
            || is_binary
    }

    #[must_use]
    #[inline]
    fn is_identifier_boundary(&self) -> bool {
        self.is_alpha(self.peek())
            || self.peek().is_ascii_digit()
            || self.peek() == b'!' && self.peek() != b':'
    }

    #[must_use]
    #[inline]
    fn is_char_boundary(&self) -> bool {
        self.peek() != b'\'' && !self.end()
    }

    #[must_use]
    #[inline]
    fn end(&self) -> bool {
        self.current >= self.code.len()
    }

    #[must_use]
    #[inline]
    fn is_alpha(&self, byte: u8) -> bool {
        byte.is_ascii_lowercase() || byte.is_ascii_uppercase() || byte == b'_'
    }

    fn add_error(&mut self, error: ThrushCompilerIssue) {
        self.errors.push(error);
    }
}
