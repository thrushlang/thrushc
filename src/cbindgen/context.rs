use crate::core::console::logging::LoggingType;
use crate::core::diagnostic::diagnostician::Diagnostician;
use crate::core::diagnostic::span::Span;
use crate::core::errors::position::CompilationPosition;
use crate::core::errors::standard::CompilationIssue;

use crate::front_end::abort;
use crate::front_end::lexer::token::Token;
use crate::front_end::lexer::tokentype::TokenType;
use crate::front_end::preprocessor::errors::PreprocessorIssue;
use crate::front_end::types::parser::stmts::traits::TokenExtensions;

#[derive(Debug)]
pub struct CBindgenContext<'cbindgen> {
    tokens: &'cbindgen [Token],
    errors: Vec<CompilationIssue>,
    module_errors: Vec<PreprocessorIssue>,
    diagnostician: Diagnostician,

    current: usize,
}

impl<'cbindgen> CBindgenContext<'cbindgen> {
    #[inline]
    pub fn new(tokens: &'cbindgen [Token], diagnostician: Diagnostician) -> Self {
        Self {
            tokens,
            errors: Vec::with_capacity(100),
            module_errors: Vec::with_capacity(100),
            diagnostician,
            current: 0,
        }
    }
}

impl<'cbindgen> CBindgenContext<'cbindgen> {
    pub fn verify(&mut self) -> Result<(), ()> {
        if !self.errors.is_empty() || !self.module_errors.is_empty() {
            self.errors.iter().for_each(|error: &CompilationIssue| {
                self.diagnostician
                    .dispatch_diagnostic(error, LoggingType::Error);
            });

            self.module_errors.iter().for_each(|error| {
                self.diagnostician
                    .dispatch_preprocessor_diagnostic(error, LoggingType::Error);
            });

            return Err(());
        }

        Ok(())
    }
}

impl<'cbindgen> CBindgenContext<'cbindgen> {
    #[inline]
    pub fn add_error(&mut self, error: CompilationIssue) {
        self.errors.push(error);
    }
}

impl<'cbindgen> CBindgenContext<'cbindgen> {
    #[inline]
    pub fn merge_module_errors(&mut self, other: Vec<PreprocessorIssue>) {
        self.module_errors.extend(other);
    }
}

impl<'cbindgen> CBindgenContext<'cbindgen> {
    #[inline]
    pub fn consume(&mut self, kind: TokenType) -> Result<&'cbindgen Token, ()> {
        if self.peek().kind == kind {
            return self.advance();
        }

        Err(())
    }
}
impl<'cbindgen> CBindgenContext<'cbindgen> {
    #[must_use]
    pub fn peek(&mut self) -> &'cbindgen Token {
        self.tokens.get(self.current).unwrap_or_else(|| {
            let span: Span = self.previous().get_span();

            abort::abort_front_end(
                self.get_mut_diagnostician(),
                CompilationPosition::Parser,
                "Unable to get a lexical token!",
                span,
                std::path::PathBuf::from(file!()),
                line!(),
            )
        })
    }

    #[must_use]
    pub fn previous(&mut self) -> &'cbindgen Token {
        self.tokens.get(self.current - 1).unwrap_or_else(|| {
            let span: Span = self.previous().get_span();

            abort::abort_front_end(
                self.get_mut_diagnostician(),
                CompilationPosition::Parser,
                "Unable to get a lexical token!",
                span,
                std::path::PathBuf::from(file!()),
                line!(),
            )
        })
    }
}

impl<'cbindgen> CBindgenContext<'cbindgen> {
    #[must_use]
    pub fn check(&mut self, kind: TokenType) -> bool {
        if self.is_eof() {
            return false;
        }

        self.peek().kind == kind
    }

    #[must_use]
    pub fn check_to(&mut self, kind: TokenType, modifier: usize) -> bool {
        if self.is_eof() {
            return false;
        }

        if self.current + modifier >= self.tokens.len() {
            return false;
        }

        self.tokens[self.current + modifier].kind == kind
    }
}

impl<'cbindgen> CBindgenContext<'cbindgen> {
    #[inline]
    pub fn match_token(&mut self, kind: TokenType) -> Result<bool, ()> {
        if self.peek().kind == kind {
            self.only_advance()?;
            return Ok(true);
        }

        Ok(false)
    }
}

impl<'cbindgen> CBindgenContext<'cbindgen> {
    #[inline]
    pub fn only_advance(&mut self) -> Result<(), ()> {
        if !self.is_eof() {
            self.current += 1;
            return Ok(());
        }

        Err(())
    }

    #[inline]
    pub fn advance(&mut self) -> Result<&'cbindgen Token, ()> {
        if !self.is_eof() {
            self.current += 1;
            return Ok(self.previous());
        }

        Err(())
    }
}

impl<'cbindgen> CBindgenContext<'cbindgen> {
    #[must_use]
    pub fn is_eof(&mut self) -> bool {
        self.peek().kind == TokenType::Eof
    }
}

impl CBindgenContext<'_> {
    #[inline]
    pub fn get_mut_diagnostician(&mut self) -> &mut Diagnostician {
        &mut self.diagnostician
    }
}
