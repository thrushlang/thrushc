use thrushc_diagnostician::Diagnostician;
use thrushc_errors::{CompilationIssue, CompilationPosition};
use thrushc_options::{CompilationUnit, CompilerOptions};
use thrushc_span::Span;
use thrushc_token::{Token, tokentype::TokenType, traits::TokenExtensions};

#[derive(Debug)]
pub struct PreprocessorContext<'preprocessor> {
    tokens: &'preprocessor [Token],
    options: &'preprocessor CompilerOptions,
    diagnostician: Diagnostician,
    errors: Vec<CompilationIssue>,
    current: usize,
}

impl<'preprocessor> PreprocessorContext<'preprocessor> {
    pub fn new(
        tokens: &'preprocessor [Token],
        options: &'preprocessor CompilerOptions,
        file: &CompilationUnit,
    ) -> Self {
        Self {
            tokens,
            options,
            diagnostician: Diagnostician::new(file, options),
            errors: Vec::with_capacity(100),
            current: 0,
        }
    }
}

impl PreprocessorContext<'_> {
    pub fn check_status(&mut self) -> Result<(), ()> {
        if !self.errors.is_empty() {
            {
                for error in self.errors.iter() {
                    self.diagnostician
                        .dispatch_diagnostic(error, thrushc_logging::LoggingType::Error);
                }
            }
        }

        Ok(())
    }
}

impl PreprocessorContext<'_> {
    #[inline]
    pub fn consume(&mut self, kind: TokenType) -> Result<&Token, ()> {
        if self.peek().kind == kind {
            return self.advance();
        }

        Err(())
    }
}

impl<'module_parser> PreprocessorContext<'module_parser> {
    #[must_use]
    pub fn peek(&mut self) -> &Token {
        self.tokens.get(self.current).unwrap_or_else(|| {
            thrushc_frontend_abort::abort_compilation(
                &mut self.diagnostician,
                CompilationPosition::Parser,
                "Unable to get current a lexical token!",
                Span::void(),
                std::path::PathBuf::from(file!()),
                line!(),
            )
        })
    }

    #[must_use]
    pub fn previous(&mut self) -> &Token {
        let index: (usize, bool) = self.current.overflowing_sub(1);

        let is_overflow: bool = index.1;
        let idx: usize = index.0;

        if is_overflow {
            let span: Span = self.peek().get_span();

            thrushc_frontend_abort::abort_compilation(
                &mut self.diagnostician,
                CompilationPosition::Parser,
                "Unable to parse previous token position!",
                span,
                std::path::PathBuf::from(file!()),
                line!(),
            )
        } else {
            let span: Span = self.peek().get_span();

            self.tokens.get(idx).unwrap_or_else(|| {
                thrushc_frontend_abort::abort_compilation(
                    &mut self.diagnostician,
                    CompilationPosition::Parser,
                    "Unable to get a lexical token!",
                    span,
                    std::path::PathBuf::from(file!()),
                    line!(),
                )
            })
        }
    }
}

impl PreprocessorContext<'_> {
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

impl PreprocessorContext<'_> {
    #[inline]
    pub fn match_token(&mut self, kind: TokenType) -> Result<bool, ()> {
        if self.peek().kind == kind {
            self.only_advance()?;
            return Ok(true);
        }

        Ok(false)
    }
}

impl PreprocessorContext<'_> {
    #[inline]
    pub fn advance_until(&mut self, kind: TokenType) -> Result<(), ()> {
        while !self.match_token(kind)? {
            self.only_advance()?;
        }

        Ok(())
    }

    #[inline]
    pub fn advance_until_check(&mut self, kind: TokenType) -> Result<(), ()> {
        while !self.check(kind) {
            self.only_advance()?;
        }

        Ok(())
    }

    #[inline]
    pub fn advance_until_limits(&mut self, limits: &[TokenType]) -> Result<(), ()> {
        while !limits.iter().any(|limit| self.check(*limit)) {
            self.only_advance()?;
        }

        self.only_advance()?;

        Ok(())
    }

    #[inline]
    pub fn advance_until_check_limits(&mut self, limits: &[TokenType]) -> Result<(), ()> {
        while !limits.iter().any(|limit| self.check(*limit)) {
            self.only_advance()?;
        }

        Ok(())
    }
}

impl PreprocessorContext<'_> {
    #[inline]
    pub fn only_advance(&mut self) -> Result<(), ()> {
        if !self.is_eof() {
            self.current += 1;
            return Ok(());
        }

        Err(())
    }

    #[inline]
    pub fn advance(&mut self) -> Result<&Token, ()> {
        if !self.is_eof() {
            self.current += 1;
            return Ok(self.previous());
        }

        Err(())
    }
}

impl PreprocessorContext<'_> {
    #[must_use]
    pub fn is_eof(&mut self) -> bool {
        self.peek().kind == TokenType::Eof
    }
}

impl PreprocessorContext<'_> {
    #[inline]
    pub fn merge_errors(&mut self, other: Vec<CompilationIssue>) {
        self.errors.extend(other);
    }
}

impl<'module_parser> PreprocessorContext<'module_parser> {
    #[inline]
    pub fn get_options(&self) -> &'module_parser CompilerOptions {
        self.options
    }
}
