pub mod atomic;
pub mod attributes;
pub mod builtins;
pub mod keywords;
pub mod printer;
pub mod scapes;
pub mod span;
pub mod token;
pub mod tokenattr;
pub mod tokenis;
pub mod tokentype;
pub mod types;

mod character;
mod identifier;
mod lex;
mod number;
mod string;

use std::{mem, process};

use unicode_categories::UnicodeCategories;

use crate::core::compiler::options::CompilationUnit;
use crate::core::console::logging::{self, LoggingType};
use crate::core::diagnostic::diagnostician::Diagnostician;
use crate::core::errors::lexer::ThrushLexerPanic;
use crate::core::errors::standard::ThrushCompilerIssue;

use crate::front_end::lexer::span::Span;
use crate::front_end::lexer::token::Token;
use crate::front_end::lexer::tokentype::TokenType;
use crate::front_end::types::lexer::types::Tokens;

const MAXIMUM_TOKENS_CAPACITY: usize = 1_000_000;
const MAXIMUM_BYTES_TO_LEX: usize = 1_000_000;
const MAX_ERRORS: usize = 50;

#[derive(Debug)]
pub struct Lexer {
    tokens: Vec<Token>,
    errors: Vec<ThrushCompilerIssue>,
    code: Vec<char>,
    bytes: Vec<u8>,

    start: usize,
    current: usize,
    line: usize,
    span: (usize, usize),

    diagnostician: Diagnostician,
}

impl Lexer {
    pub fn lex(file: &CompilationUnit) -> Result<Tokens, ThrushLexerPanic> {
        let code: Vec<char> = file.get_unit_content().chars().collect();
        let bytes: Vec<u8> = file.get_unit_content().as_bytes().to_vec();

        Self {
            tokens: Vec::with_capacity(MAXIMUM_TOKENS_CAPACITY),
            errors: Vec::with_capacity(100),
            code,
            bytes,
            start: 0,
            current: 0,
            line: 1,
            span: (0, 0),
            diagnostician: Diagnostician::new(file),
        }
        .start()
    }
}

impl Lexer {
    fn start(&mut self) -> Result<Tokens, ThrushLexerPanic> {
        if self.code.len() > MAXIMUM_BYTES_TO_LEX {
            return Err(ThrushLexerPanic::TooBigFile(
                self.diagnostician.get_file_path(),
            ));
        }

        while !self.is_eof() {
            if self.tokens.len() >= MAXIMUM_TOKENS_CAPACITY {
                return Err(ThrushLexerPanic::TooMuchTokens);
            }

            self.start = self.current;
            self.start_span();

            let total_issues: usize = self.errors.len();

            if total_issues >= MAX_ERRORS {
                logging::print_warn(
                    LoggingType::Warning,
                    "Too many issues. Stopping compilation.",
                );

                break;
            }

            if let Err(error) = lex::analyze(self) {
                self.add_error(error);
            }
        }

        if !self.errors.is_empty() {
            self.errors.iter().for_each(|error| {
                self.diagnostician
                    .dispatch_diagnostic(error, LoggingType::Error);
            });

            process::exit(1);
        };

        self.tokens.push(Token {
            lexeme: String::default(),
            bytes: Vec::default(),
            ascii: String::default(),
            kind: TokenType::Eof,
            span: Span::new(self.line, self.span),
        });

        Ok(mem::take(&mut self.tokens))
    }
}

impl Lexer {
    pub fn make(&mut self, kind: TokenType) {
        self.end_span();

        let span: Span = Span::new(self.line, self.span);

        let lexeme: String = self.lexeme();
        let bytes: Vec<u8> = self.lexeme_bytes();

        let ascii: String = if kind.is_identifier() {
            self.as_ascii_lexeme(&lexeme)
        } else {
            String::default()
        };

        self.tokens.push(Token {
            lexeme,
            ascii,
            bytes,
            kind,
            span,
        });
    }

    #[must_use]
    pub fn char_match(&mut self, char: char) -> bool {
        if !self.is_eof() && self.code[self.current] == char {
            self.current += 1;
            return true;
        }

        false
    }

    #[must_use]
    pub fn advance(&mut self) -> char {
        let byte: char = self.code[self.current];
        self.current += 1;

        byte
    }

    #[inline]
    pub fn advance_only(&mut self) {
        self.current += 1;
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
        if self.is_eof() {
            return '\0';
        }

        self.code[self.current]
    }

    #[inline]
    pub fn add_error(&mut self, error: ThrushCompilerIssue) {
        self.errors.push(error);
    }
}

impl Lexer {
    #[must_use]
    pub fn as_ascii_lexeme(&self, lexeme: &str) -> String {
        let mut scaped_unicode_string: String = String::with_capacity(lexeme.len());

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
}

impl Lexer {
    #[inline]
    pub fn lexeme(&self) -> String {
        if let Some(chars) = self.code.get(self.start..self.current) {
            return String::from_iter(chars);
        }

        logging::print_warn(
            LoggingType::Warning,
            "Couldn't get some lexeme at lexical analysis phase.",
        );

        String::default()
    }

    #[inline]
    pub fn shrink_lexeme(&self) -> String {
        if let Some(chars) = self.code.get(self.start + 1..self.current - 1) {
            return String::from_iter(chars);
        }

        logging::print_warn(
            LoggingType::Warning,
            "Couldn't shrink some lexeme at lexical analysis phase.",
        );

        String::default()
    }

    #[inline]
    pub fn shrink_lexeme_bytes(&self) -> Vec<u8> {
        if let Some(bytes) = self.bytes.get(self.start + 1..self.current - 1) {
            return bytes.to_vec();
        }

        logging::print_warn(
            LoggingType::Warning,
            "Couldn't shrink some lexeme bytes at lexical analysis phase.",
        );

        Vec::default()
    }

    #[inline]
    pub fn lexeme_bytes(&self) -> Vec<u8> {
        if let Some(bytes) = self.bytes.get(self.start..self.current) {
            return bytes.to_vec();
        }

        logging::print_warn(
            LoggingType::Warning,
            "Couldn't get lexeme bytes at lexical analysis phase.",
        );

        Vec::default()
    }
}

impl Lexer {
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

    #[must_use]
    #[inline]
    pub fn is_ascii_char(&self, peeked: char) -> bool {
        self.is_alpha_char(peeked) || peeked.is_ascii_digit() || peeked == '_' || peeked == '@'
    }

    #[must_use]
    #[inline]
    pub fn is_alpha_char(&self, char: char) -> bool {
        char.is_ascii_lowercase() || char.is_ascii_uppercase()
    }

    #[must_use]
    #[inline]
    pub fn is_eof(&self) -> bool {
        self.current >= self.code.len()
    }
}

impl Lexer {
    #[inline]
    pub fn start_span(&mut self) {
        self.span.0 = self.start;
    }

    #[inline]
    pub fn end_span(&mut self) {
        self.span.1 = self.current;
    }
}
