use crate::core::compiler::options::{CompilationUnit, CompilerOptions};
use crate::core::console::logging;
use crate::core::console::logging::LoggingType;

use crate::front_end::lexer::token::Token;
use crate::front_end::lexer::tokentype::TokenType;
use crate::front_end::preprocessor::declarations;
use crate::front_end::preprocessor::errors::PreprocessorIssue;
use crate::front_end::preprocessor::module::Module;
use crate::front_end::preprocessor::signatures::ExternalSymbol;
use crate::front_end::preprocessor::table::ModuleSymbolTable;
use crate::front_end::types::parser::stmts::traits::TokenExtensions;

#[derive(Debug)]
pub struct ModuleParser<'module_parser> {
    symbols: Vec<ExternalSymbol>,
    tokens: Vec<Token>,
    table: ModuleSymbolTable,
    options: &'module_parser CompilerOptions,

    errors: Vec<PreprocessorIssue>,

    current: usize,
}

impl<'module_parser> ModuleParser<'module_parser> {
    #[inline]
    pub fn new(tokens: Vec<Token>, options: &'module_parser CompilerOptions) -> Self {
        Self {
            symbols: Vec::with_capacity(100_000),
            options,
            table: ModuleSymbolTable::new(),
            tokens,

            errors: Vec::with_capacity(100),

            current: 0,
        }
    }
}

impl ModuleParser<'_> {
    pub fn parse(&mut self, file: CompilationUnit) -> Result<Module, Vec<PreprocessorIssue>> {
        let mut module: Module = Module::new(file);

        while !self.is_eof() {
            match self.peek().get_type() {
                TokenType::Const => {
                    if let Ok(Some(symbol)) = declarations::constant::build_constant(self) {
                        self.add_symbol(symbol);
                    }
                }

                TokenType::Import => {
                    let _ = declarations::import::build_import(self, &mut module);
                }

                _ => {
                    let _ = self.only_advance();
                }
            }
        }

        module.append_symbols(&mut self.symbols);

        if !self.errors.is_empty() {
            return Err(std::mem::take(&mut self.errors));
        }

        Ok(module)
    }
}

impl ModuleParser<'_> {
    #[inline]
    pub fn get_table(&self) -> &ModuleSymbolTable {
        &self.table
    }
}

impl ModuleParser<'_> {
    #[inline]
    pub fn add_symbol(&mut self, symbol: ExternalSymbol) {
        self.symbols.push(symbol);
    }

    #[inline]
    pub fn add_error(&mut self, error: PreprocessorIssue) {
        self.errors.push(error);
    }
}

impl ModuleParser<'_> {
    #[inline]
    pub fn merge_errors(&mut self, errors: Vec<PreprocessorIssue>) {
        self.errors.extend(errors);
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
impl ModuleParser<'_> {
    #[must_use]
    pub fn peek(&self) -> &Token {
        self.tokens.get(self.current).unwrap_or_else(|| {
            logging::print_frontend_panic(
                LoggingType::FrontEndPanic,
                "Attempting to get token in invalid current position.",
            );
        })
    }

    #[must_use]
    pub fn previous(&self) -> &Token {
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

impl ModuleParser<'_> {
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
    pub fn is_eof(&self) -> bool {
        self.peek().kind == TokenType::Eof
    }
}

impl ModuleParser<'_> {
    #[inline]
    pub fn get_options(&self) -> &CompilerOptions {
        self.options
    }
}
