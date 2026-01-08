use thrushc_diagnostician::Diagnostician;
use thrushc_errors::{CompilationIssue, CompilationPosition};
use thrushc_options::{CompilationUnit, CompilerOptions};
use thrushc_span::Span;
use thrushc_token::{Token, tokentype::TokenType, traits::TokenExtensions};

use crate::{modparsing, module::Module};

#[derive(Debug)]
pub struct ModuleParser<'module_parser> {
    module: Module<'module_parser>,
    tokens: Vec<Token>,
    diagnostician: Diagnostician,
    errors: Vec<CompilationIssue>,
    options: &'module_parser CompilerOptions,
    current: usize,
}

impl<'module_parser> ModuleParser<'module_parser> {
    pub fn new(
        name: String,
        tokens: Vec<Token>,
        options: &'module_parser CompilerOptions,
        file: &CompilationUnit,
    ) -> Self {
        Self {
            module: Module::new(name),
            tokens,
            diagnostician: Diagnostician::new(file, options),
            errors: Vec::with_capacity(255),
            options,
            current: 0,
        }
    }
}

impl<'module_parser> ModuleParser<'module_parser> {
    pub fn parse(mut self) -> Result<Module<'module_parser>, Vec<CompilationIssue>> {
        while !self.is_eof() {
            if self.start().is_err() {}

            let _ = self.only_advance();
        }

        if !self.errors.is_empty() {
            return Err(self.errors);
        }

        Ok(self.module)
    }
}

impl<'module_parser> ModuleParser<'module_parser> {
    pub fn start(&mut self) -> Result<(), ()> {
        if self.check(TokenType::Import) {
            modparsing::import::parse_import(self)?;
        }

        Ok(())
    }
}

impl ModuleParser<'_> {
    #[inline]
    pub fn add_error(&mut self, error: CompilationIssue) {
        self.errors.push(error);
    }
}

impl ModuleParser<'_> {
    #[inline]
    pub fn consume(&mut self, kind: TokenType) -> Result<&Token, ()> {
        if self.peek().kind == kind {
            return self.advance();
        }

        Err(())
    }
}

impl<'module_parser> ModuleParser<'module_parser> {
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

impl ModuleParser<'_> {
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

impl ModuleParser<'_> {
    #[inline]
    pub fn match_token(&mut self, kind: TokenType) -> Result<bool, ()> {
        if self.peek().kind == kind {
            self.only_advance()?;
            return Ok(true);
        }

        Ok(false)
    }
}

impl ModuleParser<'_> {
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

impl ModuleParser<'_> {
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

impl ModuleParser<'_> {
    #[must_use]
    pub fn is_eof(&mut self) -> bool {
        self.peek().kind == TokenType::Eof
    }
}

impl<'module_parser> ModuleParser<'module_parser> {
    #[inline]
    pub fn get_mut_module(&mut self) -> &mut Module<'module_parser> {
        &mut self.module
    }

    #[inline]
    pub fn get_module(&self) -> &Module<'module_parser> {
        &self.module
    }
}

impl<'module_parser> ModuleParser<'module_parser> {
    #[inline]
    pub fn merge_errors(&mut self, other: Vec<CompilationIssue>) {
        self.errors.extend(other);
    }
}

impl<'module_parser> ModuleParser<'module_parser> {
    #[inline]
    pub fn get_options(&self) -> &'module_parser CompilerOptions {
        self.options
    }
}
