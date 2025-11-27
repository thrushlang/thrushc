use crate::{
    core::{
        console::logging::{self, LoggingType},
        diagnostic::diagnostician::Diagnostician,
        errors::standard::CompilationIssue,
    },
    front_end::{
        lexer::{token::Token, tokentype::TokenType},
        preprocessor::errors::PreprocessorIssue,
    },
};

#[derive(Debug)]
pub struct PreprocessorContext<'preprocessor> {
    tokens: &'preprocessor [Token],
    errors: Vec<CompilationIssue>,
    module_errors: Vec<PreprocessorIssue>,
    diagnostician: Diagnostician,

    current: usize,
}

impl<'preprocessor> PreprocessorContext<'preprocessor> {
    #[inline]
    pub fn new(tokens: &'preprocessor [Token], diagnostician: Diagnostician) -> Self {
        Self {
            tokens,
            errors: Vec::with_capacity(100),
            module_errors: Vec::with_capacity(100),
            diagnostician,
            current: 0,
        }
    }
}

impl<'preprocessor> PreprocessorContext<'preprocessor> {
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

impl<'preprocessor> PreprocessorContext<'preprocessor> {
    #[inline]
    pub fn add_error(&mut self, error: CompilationIssue) {
        self.errors.push(error);
    }
}

impl<'preprocessor> PreprocessorContext<'preprocessor> {
    #[inline]
    pub fn merge_module_errors(&mut self, other: Vec<PreprocessorIssue>) {
        self.module_errors.extend(other);
    }
}

impl<'preprocessor> PreprocessorContext<'preprocessor> {
    #[inline]
    pub fn consume(&mut self, kind: TokenType) -> Result<&'preprocessor Token, ()> {
        if self.peek().kind == kind {
            return self.advance();
        }

        Err(())
    }
}
impl<'preprocessor> PreprocessorContext<'preprocessor> {
    #[must_use]
    pub fn peek(&self) -> &'preprocessor Token {
        self.tokens.get(self.current).unwrap_or_else(|| {
            logging::print_frontend_panic(
                LoggingType::FrontEndPanic,
                "Attempting to get token in invalid current position.",
            );
        })
    }

    #[must_use]
    pub fn previous(&self) -> &'preprocessor Token {
        self.tokens.get(self.current - 1).unwrap_or_else(|| {
            logging::print_frontend_panic(
                LoggingType::FrontEndPanic,
                &format!(
                    "Attempting to get token in invalid previous position in line '{}'.",
                    self.peek().span.get_line()
                ),
            );
        })
    }
}

impl<'preprocessor> PreprocessorContext<'preprocessor> {
    #[must_use]
    pub fn check(&self, kind: TokenType) -> bool {
        if self.is_eof() {
            return false;
        }

        self.peek().kind == kind
    }

    #[must_use]
    pub fn check_to(&self, kind: TokenType, modifier: usize) -> bool {
        if self.is_eof() {
            return false;
        }

        if self.current + modifier >= self.tokens.len() {
            return false;
        }

        self.tokens[self.current + modifier].kind == kind
    }
}

impl<'preprocessor> PreprocessorContext<'preprocessor> {
    #[inline]
    pub fn match_token(&mut self, kind: TokenType) -> Result<bool, ()> {
        if self.peek().kind == kind {
            self.only_advance()?;
            return Ok(true);
        }

        Ok(false)
    }
}

impl<'preprocessor> PreprocessorContext<'preprocessor> {
    #[inline]
    pub fn only_advance(&mut self) -> Result<(), ()> {
        if !self.is_eof() {
            self.current += 1;
            return Ok(());
        }

        Err(())
    }

    #[inline]
    pub fn advance(&mut self) -> Result<&'preprocessor Token, ()> {
        if !self.is_eof() {
            self.current += 1;
            return Ok(self.previous());
        }

        Err(())
    }
}

impl<'preprocessor> PreprocessorContext<'preprocessor> {
    #[must_use]
    pub fn is_eof(&self) -> bool {
        self.peek().kind == TokenType::Eof
    }
}
