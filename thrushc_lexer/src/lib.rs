use thrushc_diagnostician::Diagnostician;
use thrushc_errors::CompilationIssue;
use thrushc_logging::LoggingType;
use thrushc_options::{CompilationUnit, CompilerOptions};
use thrushc_span::Span;
use thrushc_token::{Token, tokentype::TokenType, traits::TokenTypeExtensions};
use unicode_categories::UnicodeCategories;

const PREALLOCATED_TOKENS_CAPACITY: usize = 10_000;

mod character;
mod identifier;
mod lex;
mod number;
pub mod printer;
mod string;

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
    pub fn lex(file: &CompilationUnit, options: &CompilerOptions) -> Vec<Token> {
        let code: Vec<char> = file.get_unit_content().chars().collect();
        let bytes: Vec<u8> = file.get_unit_content().as_bytes().to_vec();

        Self {
            tokens: Vec::with_capacity(PREALLOCATED_TOKENS_CAPACITY),
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
    fn start(&mut self) -> Vec<Token> {
        while !self.is_eof() {
            self.start = self.current;
            self.start_span();

            if let Err(error) = lex::analyze(self) {
                self.add_error(error);
            }
        }

        if !self.errors.is_empty() {
            self.errors.iter().for_each(|error| {
                self.diagnostician
                    .dispatch_diagnostic(error, LoggingType::Error);
            });

            std::process::exit(1);
        } else {
            self.tokens.push(Token {
                lexeme: String::default(),
                bytes: Vec::default(),
                ascii: String::default(),
                kind: TokenType::Eof,
                span: Span::new(self.line, self.span),
            });

            std::mem::take(&mut self.tokens)
        }
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

        thrushc_logging::print_warn(
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

        thrushc_logging::print_warn(
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

        thrushc_logging::print_warn(
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

        thrushc_logging::print_warn(
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
