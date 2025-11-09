use std::mem;

use crate::{
    core::{
        compiler::options::CompilationUnit,
        console::logging::{self, LoggingType},
    },
    front_end::{
        lexer::{token::Token, tokentype::TokenType},
        preprocessor::{
            declarations, errors::PreprocessorIssue, module::Module, signatures::ExternalSymbol,
            table::ModuleSymbolTable,
        },
        types::parser::stmts::traits::TokenExtensions,
    },
};

#[derive(Debug)]
pub struct ModuleParser<'module_parser> {
    symbols: Vec<ExternalSymbol<'module_parser>>,
    tokens: Vec<Token>,
    errors: Vec<PreprocessorIssue>,
    table: ModuleSymbolTable,

    current: usize,
}

impl<'module_parser> ModuleParser<'module_parser> {
    #[inline]
    pub fn new(tokens: Vec<Token>) -> Self {
        Self {
            symbols: Vec::with_capacity(100_000),
            table: ModuleSymbolTable::new(),
            tokens,
            errors: Vec::with_capacity(100),

            current: 0,
        }
    }
}

impl<'module_parser> ModuleParser<'module_parser> {
    pub fn parse(
        &mut self,
        file: CompilationUnit,
    ) -> Result<Module<'module_parser>, Vec<PreprocessorIssue>> {
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
            return Err(mem::take(&mut self.errors));
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

impl<'module_parser> ModuleParser<'module_parser> {
    #[inline]
    pub fn add_symbol(&mut self, symbol: ExternalSymbol<'module_parser>) {
        self.symbols.push(symbol);
    }

    #[inline]
    pub fn add_error(&mut self, error: PreprocessorIssue) {
        self.errors.push(error);
    }
}

impl<'module_parser> ModuleParser<'module_parser> {
    #[inline]
    pub fn merge_errors(&mut self, errors: Vec<PreprocessorIssue>) {
        self.errors.extend(errors);
    }
}

impl<'module_parser> ModuleParser<'module_parser> {
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

impl<'module_parser> ModuleParser<'module_parser> {
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

impl<'module_parser> ModuleParser<'module_parser> {
    #[inline]
    pub fn match_token(&mut self, kind: TokenType) -> Result<bool, ()> {
        if self.peek().kind == kind {
            self.only_advance()?;
            return Ok(true);
        }

        Ok(false)
    }
}

impl<'module_parser> ModuleParser<'module_parser> {
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

impl<'module_parser> ModuleParser<'module_parser> {
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

impl<'module_parser> ModuleParser<'module_parser> {
    #[must_use]
    pub fn is_eof(&self) -> bool {
        self.peek().kind == TokenType::Eof
    }
}
