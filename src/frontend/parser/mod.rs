pub mod attributes;
pub mod builtins;
pub mod contexts;
pub mod declarations;
pub mod expression;
pub mod expressions;
pub mod parse;
pub mod stmt;
pub mod stmts;
pub mod symbols;
pub mod typegen;

use ahash::AHashMap as HashMap;

use contexts::{ParserControlContext, ParserTypeContext, SyncPosition};
use symbols::SymbolsTable;

use crate::core::compiler::options::CompilerFile;
use crate::core::console::logging::{self, LoggingType};
use crate::core::diagnostic::diagnostician::Diagnostician;
use crate::core::errors::standard::ThrushCompilerIssue;
use crate::frontend::lexer::token::Token;
use crate::frontend::lexer::tokentype::TokenType;
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
        let mut parser_ctx: ParserContext = ParserContext::new(self.tokens, self.file);

        parser_ctx.forward();

        while !parser_ctx.is_eof() {
            match stmt::parse(&mut parser_ctx) {
                Ok(instr) => {
                    parser_ctx.add_stmt(instr);
                }
                Err(error) => {
                    parser_ctx.add_error(error);
                    parser_ctx.sync();
                }
            }
        }

        let throwed_errors: bool = parser_ctx.verify();

        (parser_ctx, throwed_errors)
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
                while !self.is_eof() && !self.peek().kind.is_sync_declaration() {
                    self.current += 1;
                }

                self.scope = 0;
                self.symbols.clear_all_scopes();
            }
            SyncPosition::Statement => {
                if let Some((lbrace_count, rbrace_count, diff)) = self.get_peerless_scopes() {
                    if lbrace_count != rbrace_count {
                        for _ in 0..diff {
                            self.scope = self.scope.saturating_sub(1);
                            self.symbols.end_scope();
                        }
                    }
                }

                while !self.is_eof()
                    && !self.peek().kind.is_sync_declaration()
                    && !self.peek().kind.is_sync_statement()
                {
                    self.current += 1;
                }
            }
            SyncPosition::Expression => {
                if let Some((lbrace_count, rbrace_count, diff)) = self.get_peerless_scopes() {
                    if lbrace_count != rbrace_count {
                        for _ in 0..diff {
                            self.scope = self.scope.saturating_sub(1);
                            self.symbols.end_scope();
                        }
                    }
                }

                while !self.is_eof() {
                    match self.peek().kind {
                        any if any.is_sync_expression()
                            || any.is_sync_statement()
                            || any.is_sync_declaration() =>
                        {
                            self.current += 1;

                            if self.peek().kind.is_sync_expression() {
                                continue;
                            }

                            break;
                        }

                        _ => (),
                    }

                    self.current += 1;
                }
            }
            _ => {}
        }

        self.control_ctx.set_sync_position(SyncPosition::NoRelevant);

        self.symbols.end_parameters();

        self.control_ctx.set_inside_function(false);
        self.control_ctx.reset_loop_depth();
    }

    pub fn is_unreacheable_code(&self) -> bool {
        self.control_ctx.get_unreacheable_code_scope() == self.scope && !self.is_main_scope()
    }

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

    pub fn add_stmt(&mut self, stmt: Ast<'parser>) {
        self.ast.push(stmt);
    }

    pub fn add_error(&mut self, error: ThrushCompilerIssue) {
        self.errors.push(error);
    }

    pub fn get_ast(&self) -> &[Ast<'parser>] {
        &self.ast
    }

    #[must_use]
    pub fn check(&self, kind: TokenType) -> bool {
        if self.is_eof() {
            return false;
        }

        self.peek().kind == kind
    }

    #[must_use]
    pub fn check_to(&self, kind: TokenType, changer: usize) -> bool {
        if self.is_eof() {
            return false;
        }

        if self.current + changer >= self.tokens.len() {
            return false;
        }

        self.tokens[self.current + changer].kind == kind
    }

    #[must_use]
    pub const fn is_main_scope(&self) -> bool {
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

    fn get_peerless_scopes(&mut self) -> Option<(usize, usize, usize)> {
        self.tokens[self.current..]
            .iter()
            .enumerate()
            .find(|(_, tk)| tk.kind.is_sync_statement() || tk.kind.is_sync_declaration())
            .map(|(i, _)| {
                let limit_pos: usize = self.current + i;

                let lbrace_count: usize = self.tokens[self.current..limit_pos]
                    .iter()
                    .rev()
                    .filter(|tk| matches!(tk.kind, TokenType::LBrace))
                    .count();

                let rbrace_count: usize = self.tokens[self.current..limit_pos]
                    .iter()
                    .rev()
                    .filter(|tk| matches!(tk.kind, TokenType::RBrace))
                    .count();

                let diff: usize = if lbrace_count > rbrace_count {
                    lbrace_count - rbrace_count
                } else {
                    rbrace_count - lbrace_count
                };

                (lbrace_count, rbrace_count, diff)
            })
    }

    pub fn forward(&mut self) {
        stmt::parse_forward(self);
    }
}
