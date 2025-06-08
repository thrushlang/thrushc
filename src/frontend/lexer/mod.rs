use std::{mem, process};

use keywords::THRUSH_KEYWORDS;
use span::Span;
use token::Token;
use unicode_categories::UnicodeCategories;

use crate::{
    core::{
        compiler::options::CompilerFile,
        console::logging::LoggingType,
        diagnostic::diagnostician::Diagnostician,
        errors::{lexer::ThrushLexerPanic, standard::ThrushCompilerIssue},
    },
    frontend::lexer::tokentype::TokenType,
};

pub mod keywords;
pub mod span;
pub mod token;
pub mod tokentype;

const MAXIMUM_TOKENS_CAPACITY: usize = 1_000_000;
const MAXIMUM_BYTES_TO_LEX: usize = 1_000_000;

pub struct Lexer {
    tokens: Vec<Token>,
    errors: Vec<ThrushCompilerIssue>,
    code: Vec<char>,
    start: usize,
    current: usize,
    line: usize,
    span: (usize, usize),
    diagnostician: Diagnostician,
}

impl Lexer {
    pub fn lex(raw_code: &str, file: &CompilerFile) -> Result<Vec<Token>, ThrushLexerPanic> {
        let code: Vec<char> = raw_code.chars().collect();

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

    fn start(&mut self) -> Result<Vec<Token>, ThrushLexerPanic> {
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
                self.add_error(error);
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
            lexeme: String::new(),
            ascii_lexeme: String::new(),
            kind: TokenType::Eof,
            span: Span::new(self.line, self.span),
        });

        Ok(mem::take(&mut self.tokens))
    }

    fn analyze(&mut self) -> Result<(), ThrushCompilerIssue> {
        match self.advance() {
            '[' => self.make(TokenType::LBracket),
            ']' => self.make(TokenType::RBracket),
            '(' => self.make(TokenType::LParen),
            ')' => self.make(TokenType::RParen),
            '{' => self.make(TokenType::LBrace),
            '}' => self.make(TokenType::RBrace),
            ',' => self.make(TokenType::Comma),
            '.' if self.char_match('.') && self.char_match('.') => self.make(TokenType::Pass),
            '.' if self.char_match('.') => self.make(TokenType::Range),
            '.' => self.make(TokenType::Dot),
            '%' => self.make(TokenType::Arith),
            '*' => self.make(TokenType::Star),
            '/' if self.char_match('/') => loop {
                if self.peek() == '\n' || self.end() {
                    break;
                }

                self.advance();
            },
            '/' if self.char_match('*') => loop {
                if self.char_match('*') && self.char_match('/') {
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
            '/' => self.make(TokenType::Slash),
            ';' => self.make(TokenType::SemiColon),
            '-' if self.char_match('-') => self.make(TokenType::MinusMinus),
            '-' if self.char_match('=') => self.make(TokenType::MinusEq),
            '-' if self.char_match('>') => self.make(TokenType::Arrow),
            '-' => self.make(TokenType::Minus),
            '+' if self.char_match('+') => self.make(TokenType::PlusPlus),
            '+' if self.char_match('=') => self.make(TokenType::PlusEq),
            '+' => self.make(TokenType::Plus),
            ':' if self.char_match(':') => self.make(TokenType::ColonColon),
            ':' => self.make(TokenType::Colon),
            '!' if self.char_match('=') => self.make(TokenType::BangEq),
            '!' => self.make(TokenType::Bang),
            '=' if self.char_match('=') => self.make(TokenType::EqEq),
            '=' => self.make(TokenType::Eq),
            '<' if self.char_match('=') => self.make(TokenType::LessEq),
            '<' if self.char_match('<') => self.make(TokenType::LShift),
            '<' => self.make(TokenType::Less),
            '>' if self.char_match('=') => self.make(TokenType::GreaterEq),
            '>' if self.char_match('>') => self.make(TokenType::RShift),
            '>' => self.make(TokenType::Greater),
            '|' if self.char_match('|') => self.make(TokenType::Or),
            '&' if self.char_match('&') => self.make(TokenType::And),
            ' ' | '\r' | '\t' => {}
            '\n' => self.line += 1,
            '\'' => self.char()?,
            '"' => self.string()?,
            '0'..='9' => self.number()?,
            identifier if self.is_identifier_boundary(identifier) => self.identifier()?,
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
        while self.is_identifier_boundary(self.peek()) {
            self.advance();
        }

        let mut lexeme: String = String::with_capacity(100);

        lexeme.extend(&self.code[self.start..self.current]);

        if let Some(keyword) = THRUSH_KEYWORDS.get(lexeme.as_str()) {
            self.make(*keyword);
        } else {
            self.make(TokenType::Identifier);
        }

        Ok(())
    }

    fn number(&mut self) -> Result<(), ThrushCompilerIssue> {
        let mut is_hexadecimal: bool = false;
        let mut is_binary: bool = false;

        while self.is_number_boundary(is_hexadecimal, is_binary) {
            if is_hexadecimal && self.previous() == '0' && self.peek() == 'x' {
                self.end_span();

                return Err(ThrushCompilerIssue::Error(
                    String::from("Syntax error"),
                    String::from("Hexadecimal identifier '0x' cannot be repeated."),
                    None,
                    Span::new(self.line, self.span),
                ));
            }

            if is_binary && self.previous() == '0' && self.peek() == 'b' {
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

            if self.peek() == 'x' && self.peek_next().is_ascii_alphanumeric() {
                is_hexadecimal = true;
            }

            if self.peek() == 'b' && self.peek_next().is_ascii_digit() {
                is_binary = true;
            }

            self.advance();
        }

        self.end_span();

        let span: Span = Span::new(self.line, self.span);

        let lexeme: String = self.lexeme();

        if lexeme.contains(".") {
            self.check_float_format(&lexeme)?;

            self.tokens.push(Token {
                lexeme,
                ascii_lexeme: String::new(),
                kind: TokenType::Float,
                span,
            });

            return Ok(());
        }

        self.check_integer_format(&lexeme)?;

        self.tokens.push(Token {
            lexeme,
            ascii_lexeme: String::new(),
            kind: TokenType::Integer,
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

        if self.peek() != '\'' {
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

        let lexeme: String = self.shrink_lexeme();

        self.tokens.push(Token {
            kind: TokenType::Char,
            ascii_lexeme: String::default(),
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

        if self.peek() != '"' {
            return Err(ThrushCompilerIssue::Error(
                String::from("Syntax error"),
                String::from("Unclosed literal string. Did you forget to close it with a '\"'?"),
                None,
                span,
            ));
        }

        self.advance();

        let lexeme: String = self.shrink_lexeme();

        self.tokens.push(Token {
            kind: TokenType::Str,
            ascii_lexeme: String::default(),
            lexeme,
            span,
        });

        Ok(())
    }

    fn make(&mut self, kind: TokenType) {
        self.end_span();

        let span: Span = Span::new(self.line, self.span);

        let lexeme: String = self.lexeme();

        let ascii_lexeme: String = if kind.is_identifier() {
            self.fix_unicode_lexeme(&lexeme)
        } else {
            String::default()
        };

        self.tokens.push(Token {
            lexeme,
            ascii_lexeme,
            kind,
            span,
        });
    }

    fn char_match(&mut self, char: char) -> bool {
        if !self.end() && self.code[self.current] == char {
            self.current += 1;
            return true;
        }

        false
    }

    fn advance(&mut self) -> char {
        let byte: char = self.code[self.current];
        self.current += 1;

        byte
    }

    #[inline]
    fn lexeme(&self) -> String {
        String::from_iter(&self.code[self.start..self.current])
    }

    #[inline]
    fn shrink_lexeme(&self) -> String {
        String::from_iter(&self.code[self.start + 1..self.current - 1])
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
    fn peek_next(&self) -> char {
        if self.current + 1 >= self.code.len() {
            return '\0';
        }

        self.code[self.current + 1]
    }

    #[must_use]
    fn previous(&self) -> char {
        self.code[self.current - 1]
    }

    #[must_use]
    #[inline]
    fn peek(&self) -> char {
        if self.end() {
            return '\0';
        }

        self.code[self.current]
    }

    #[must_use]
    #[inline]
    fn is_string_boundary(&self) -> bool {
        self.peek() != '"' && !self.end()
    }

    #[must_use]
    #[inline]
    fn is_number_boundary(&self, is_hexadecimal: bool, is_binary: bool) -> bool {
        self.peek().is_ascii_digit()
            || self.peek() == '_'
            || self.peek() == '.'
            || self.peek() == 'x'
            || self.peek() == 'b'
            || is_hexadecimal
            || is_binary
    }

    #[must_use]
    #[inline]
    fn is_identifier_boundary(&self, peeked: char) -> bool {
        peeked.is_alphanumeric()
            || peeked == '_'
            || peeked == '@'
            || (peeked == '!' && self.peek() != ':')
            || peeked.is_symbol_other()
    }

    fn fix_unicode_lexeme(&self, lexeme: &str) -> String {
        let mut fixed_unicode_string: String = String::with_capacity(500);

        lexeme.chars().for_each(|char| {
            if self.is_ascii_char(char) {
                fixed_unicode_string.push(char);
            } else {
                let mut utf8_buf: [u8; 4] = [0u8; 4];

                let utf8_bytes: &[u8] = char.encode_utf8(&mut utf8_buf).as_bytes();

                utf8_bytes.iter().for_each(|byte| {
                    fixed_unicode_string.push_str(&format!("{:02X}", byte));
                });
            }
        });

        fixed_unicode_string
    }

    fn is_ascii_char(&self, char: char) -> bool {
        self.is_alpha_char(char)
            || char.is_ascii_digit()
            || char == '!' && char != ':'
            || char == '_'
    }

    #[must_use]
    #[inline]
    fn is_char_boundary(&self) -> bool {
        self.peek() != '\'' && !self.end()
    }

    #[must_use]
    #[inline]
    fn end(&self) -> bool {
        self.current >= self.code.len()
    }

    #[must_use]
    #[inline]
    fn is_alpha_char(&self, char: char) -> bool {
        char.is_ascii_lowercase() || char.is_ascii_uppercase()
    }

    fn add_error(&mut self, error: ThrushCompilerIssue) {
        self.errors.push(error);
    }
}
