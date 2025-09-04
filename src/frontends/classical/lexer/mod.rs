use std::{mem, process};

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
    frontends::classical::{lexer::tokentype::TokenType, types::lexer::types::Tokens},
};

pub mod keywords;
pub mod printer;
pub mod span;
pub mod token;
pub mod tokentype;

mod character;
mod identifier;
mod lex;
mod number;
mod string;

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
    pub fn lex(raw_code: &str, file: &CompilerFile) -> Result<Tokens, ThrushLexerPanic> {
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

    fn start(&mut self) -> Result<Tokens, ThrushLexerPanic> {
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

            if let Err(error) = lex::analyze(self) {
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

    pub fn make(&mut self, kind: TokenType) {
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

    pub fn char_match(&mut self, char: char) -> bool {
        if !self.end() && self.code[self.current] == char {
            self.current += 1;
            return true;
        }

        false
    }

    pub fn advance(&mut self) -> char {
        let byte: char = self.code[self.current];
        self.current += 1;

        byte
    }

    #[inline]
    pub fn lexeme(&self) -> String {
        String::from_iter(&self.code[self.start..self.current])
    }

    #[inline]
    pub fn shrink_lexeme(&self) -> String {
        String::from_iter(&self.code[self.start + 1..self.current - 1])
    }

    #[inline]
    pub fn start_span(&mut self) {
        self.span.0 = self.start;
    }

    #[inline]
    pub fn end_span(&mut self) {
        self.span.1 = self.current;
    }

    #[inline]
    pub fn peek_next(&self) -> char {
        if self.current + 1 >= self.code.len() {
            return '\0';
        }

        self.code[self.current + 1]
    }

    #[must_use]
    pub fn previous(&self) -> char {
        self.code[self.current - 1]
    }

    #[must_use]
    #[inline]
    pub fn peek(&self) -> char {
        if self.end() {
            return '\0';
        }

        self.code[self.current]
    }

    #[must_use]
    #[inline]
    pub fn is_number_boundary(&self, is_hexadecimal: bool, is_binary: bool) -> bool {
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
    pub fn is_identifier_boundary(&self, peeked: char) -> bool {
        peeked.is_alphanumeric() || peeked.is_symbol_other() || peeked == '_' || peeked == '@'
    }

    pub fn fix_unicode_lexeme(&self, lexeme: &str) -> String {
        let mut scaped_unicode_string: String = String::with_capacity(100);

        lexeme.chars().for_each(|char| {
            if self.is_ascii_char(char) {
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

    pub fn is_ascii_char(&self, peeked: char) -> bool {
        self.is_alpha_char(peeked) || peeked.is_ascii_digit() || peeked == '_' || peeked == '@'
    }

    #[must_use]
    #[inline]
    pub fn is_char_boundary(&self) -> bool {
        self.peek() != '\'' && !self.end()
    }

    #[must_use]
    #[inline]
    pub fn end(&self) -> bool {
        self.current >= self.code.len()
    }

    #[must_use]
    #[inline]
    pub fn is_alpha_char(&self, char: char) -> bool {
        char.is_ascii_lowercase() || char.is_ascii_uppercase()
    }

    #[inline]
    pub fn add_error(&mut self, error: ThrushCompilerIssue) {
        self.errors.push(error);
    }
}
