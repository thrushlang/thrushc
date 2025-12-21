pub mod atomic;
pub mod attributes;
pub mod builtins;
pub mod character;
pub mod identifier;
pub mod keywords;
pub mod lex;
pub mod number;
pub mod printer;
pub mod scapes;
pub mod string;
pub mod token;
pub mod tokentype;
pub mod types;

use crate::core::compiler::options::{CompilationUnit, CompilerOptions};
use crate::core::console::logging::{self, LoggingType};
use crate::core::diagnostic::diagnostician::Diagnostician;
use crate::core::diagnostic::span::Span;
use crate::core::errors::lexer::LexerPanic;
use crate::core::errors::standard::CompilationIssue;

use crate::front_end::lexer::token::Token;
use crate::front_end::lexer::tokentype::TokenType;
use crate::front_end::types::lexer::traits::TokenTypeExtensions;
use crate::front_end::types::lexer::types::Tokens;

use std::{mem, process};

use unicode_categories::UnicodeCategories;

const MAXIMUM_TOKENS_CAPACITY: usize = 1_000_000_000;
const MAXIMUM_BYTES_TO_LEX: usize = 1_000_000;
const MAX_ERRORS: usize = 50;

#[derive(Debug)]
pub struct Lexer {
    tokens: Vec<Token>,
    errors: Vec<CompilationIssue>,
    code: Vec<char>,
    bytes: Vec<u8>,

    start: usize,
    current: usize,
    line: usize,
    span: (usize, usize),

    diagnostician: Diagnostician,
}

impl Lexer {
    pub fn lex(file: &CompilationUnit, options: &CompilerOptions) -> Result<Tokens, LexerPanic> {
        let code: Vec<char> = file.get_unit_content().chars().collect();
        let bytes: Vec<u8> = file.get_unit_content().as_bytes().to_vec();

        Self {
            tokens: Vec::with_capacity(100_000),
            errors: Vec::with_capacity(100),
            code,
            bytes,
            start: 0,
            current: 0,
            line: 1,
            span: (0, 0),
            diagnostician: Diagnostician::new(file, options),
        }
        .start()
    }
}

impl Lexer {
    fn start(&mut self) -> Result<Tokens, LexerPanic> {
        if self.code.len() > MAXIMUM_BYTES_TO_LEX {
            return Err(LexerPanic::TooBigFile(self.diagnostician.get_file_path()));
        }

        while !self.is_eof() {
            if self.tokens.len() >= MAXIMUM_TOKENS_CAPACITY {
                return Err(LexerPanic::TooMuchTokens);
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
            string::convert_to_ascii(self, &lexeme)
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
    pub fn add_error(&mut self, error: CompilationIssue) {
        self.errors.push(error);
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
