pub mod attributes;
pub mod builtins;
pub mod checks;
pub mod contexts;
pub mod declaration;
pub mod declarations;
pub mod expr;
pub mod expressions;
pub mod parse;
pub mod stmt;
pub mod stmts;
pub mod symbols;
pub mod typegen;

use ahash::AHashMap as HashMap;

use contexts::{sync::SyncPosition, typectx::ParserTypeContext};
use symbols::SymbolsTable;

use crate::core::compiler::options::CompilerFile;
use crate::core::console::logging::{self, LoggingType};
use crate::core::diagnostic::diagnostician::Diagnostician;
use crate::core::errors::standard::ThrushCompilerIssue;
use crate::frontend::lexer::token::Token;
use crate::frontend::lexer::tokentype::TokenType;
use crate::frontend::parser::contexts::controlctx::ParserControlContext;
use crate::frontend::types::ast::Ast;
use crate::frontend::types::parser::symbols::types::{AssemblerFunctions, Functions};

const MINIMAL_STATEMENT_CAPACITY: usize = 100_000;
const MINIMAL_GLOBAL_CAPACITY: usize = 2024;

pub struct ParserContext<'parser> {
    ast: Vec<Ast<'parser>>,
    tokens: &'parser [Token],
    errors: Vec<ThrushCompilerIssue>,

    control_ctx: ParserControlContext,
    type_ctx: ParserTypeContext,
    diagnostician: Diagnostician,
    symbols: SymbolsTable<'parser>,

    current: usize,
    scope: usize,
}

pub struct Parser<'parser> {
    tokens: &'parser [Token],
    file: &'parser CompilerFile,
}

impl<'parser> Parser<'parser> {
    pub fn parse(
        tokens: &'parser [Token],
        file: &'parser CompilerFile,
    ) -> (ParserContext<'parser>, bool) {
        Self { tokens, file }.start()
    }

    fn start(&mut self) -> (ParserContext<'parser>, bool) {
        let mut parser_context: ParserContext = ParserContext::new(self.tokens, self.file);

        parser_context.declare_forward();

        while !parser_context.is_eof() {
            match declaration::decl(&mut parser_context) {
                Ok(instr) => {
                    parser_context.add_stmt(instr);
                }
                Err(error) => {
                    parser_context.add_error(error);
                    parser_context.sync();
                }
            }
        }

        let throwed_errors: bool = parser_context.verify();

        (parser_context, throwed_errors)
    }
}

impl<'parser> ParserContext<'parser> {
    pub fn new(tokens: &'parser [Token], file: &'parser CompilerFile) -> Self {
        let functions: Functions = HashMap::with_capacity(MINIMAL_GLOBAL_CAPACITY);
        let asm_functions: AssemblerFunctions = HashMap::with_capacity(MINIMAL_GLOBAL_CAPACITY);

        Self {
            tokens,
            ast: Vec::with_capacity(MINIMAL_STATEMENT_CAPACITY),
            errors: Vec::with_capacity(100),
            control_ctx: ParserControlContext::new(),
            type_ctx: ParserTypeContext::new(),
            diagnostician: Diagnostician::new(file),
            symbols: SymbolsTable::with_functions(functions, asm_functions),
            current: 0,
            scope: 0,
        }
    }

    pub fn verify(&mut self) -> bool {
        if !self.errors.is_empty() {
            self.errors.iter().for_each(|error: &ThrushCompilerIssue| {
                self.diagnostician
                    .build_diagnostic(error, LoggingType::Error);
            });

            return true;
        }

        false
    }

    pub fn consume(
        &mut self,
        kind: TokenType,
        title: String,
        help: String,
    ) -> Result<&'parser Token, ThrushCompilerIssue> {
        if self.peek().kind == kind {
            return self.advance();
        }

        Err(ThrushCompilerIssue::Error(
            title,
            help,
            None,
            self.previous().span,
        ))
    }

    pub fn match_token(&mut self, kind: TokenType) -> Result<bool, ThrushCompilerIssue> {
        if self.peek().kind == kind {
            self.only_advance()?;
            return Ok(true);
        }

        Ok(false)
    }

    pub fn only_advance(&mut self) -> Result<(), ThrushCompilerIssue> {
        if !self.is_eof() {
            self.current += 1;
            return Ok(());
        }

        Err(ThrushCompilerIssue::Error(
            String::from("Syntax error"),
            String::from("EOF has been reached."),
            None,
            self.peek().span,
        ))
    }

    pub fn advance(&mut self) -> Result<&'parser Token, ThrushCompilerIssue> {
        if !self.is_eof() {
            self.current += 1;
            return Ok(self.previous());
        }

        Err(ThrushCompilerIssue::Error(
            String::from("Syntax error"),
            String::from("EOF has been reached."),
            None,
            self.peek().span,
        ))
    }

    pub fn sync(&mut self) {
        match self.control_ctx.get_sync_position() {
            SyncPosition::Declaration => {
                loop {
                    if self.is_eof() {
                        break;
                    }

                    if self.peek().kind.is_sync_declaration() {
                        break;
                    }

                    let _ = self.only_advance();
                }

                self.scope = 0;
                self.symbols.end_parameters();
            }

            SyncPosition::Statement => loop {
                if self.is_eof() {
                    break;
                }

                if self.peek().kind.is_sync_statement() || self.peek().kind.is_sync_declaration() {
                    break;
                }

                let _ = self.only_advance();
            },

            SyncPosition::Expression => loop {
                if self.is_eof() {
                    break;
                }

                if self.peek().kind.is_sync_expression()
                    || self.peek().kind.is_sync_statement()
                    || self.peek().kind.is_sync_declaration()
                {
                    break;
                }

                let _ = self.only_advance();
            },

            _ => {}
        }

        self.control_ctx.set_sync_position(SyncPosition::NoRelevant);
    }

    pub fn is_unreacheable_code(&self) -> bool {
        self.control_ctx.get_unreacheable_code_scope() == self.scope && !self.is_main_scope()
    }

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

    #[must_use]
    pub fn is_main_scope(&self) -> bool {
        self.scope == 0
    }

    #[must_use]
    pub fn is_eof(&self) -> bool {
        self.peek().kind == TokenType::Eof
    }

    #[must_use]
    pub fn peek(&self) -> &'parser Token {
        self.tokens.get(self.current).unwrap_or_else(|| {
            logging::log(
                LoggingType::Panic,
                "Attempting to get token in invalid current position.",
            );

            unreachable!()
        })
    }

    #[must_use]
    pub fn previous(&self) -> &'parser Token {
        self.tokens.get(self.current - 1).unwrap_or_else(|| {
            logging::log(
                LoggingType::Panic,
                &format!(
                    "Attempting to get token in invalid previous position in line '{}'.",
                    self.peek().span.get_line()
                ),
            );
            unreachable!()
        })
    }

    pub fn declare_forward(&mut self) {
        stmt::parse_forward(self);
    }
}

impl<'parser> ParserContext<'parser> {
    pub fn get_symbols(&self) -> &SymbolsTable<'parser> {
        &self.symbols
    }

    pub fn get_mut_symbols(&mut self) -> &mut SymbolsTable<'parser> {
        &mut self.symbols
    }

    pub fn get_control_ctx(&mut self) -> &ParserControlContext {
        &mut self.control_ctx
    }

    pub fn get_mut_control_ctx(&mut self) -> &mut ParserControlContext {
        &mut self.control_ctx
    }

    pub fn get_type_ctx(&self) -> &ParserTypeContext {
        &self.type_ctx
    }

    pub fn get_mut_type_ctx(&mut self) -> &mut ParserTypeContext {
        &mut self.type_ctx
    }

    pub fn get_scope(&self) -> usize {
        self.scope
    }

    pub fn get_mut_scope(&mut self) -> &mut usize {
        &mut self.scope
    }

    pub fn get_ast(&self) -> &[Ast<'parser>] {
        &self.ast
    }
}

impl<'parser> ParserContext<'parser> {
    pub fn add_stmt(&mut self, stmt: Ast<'parser>) {
        self.ast.push(stmt);
    }

    pub fn add_error(&mut self, error: ThrushCompilerIssue) {
        self.errors.push(error);
    }
}
