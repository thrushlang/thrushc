use either::Either;

use thrushc_ast::Ast;
use thrushc_diagnostician::Diagnostician;
use thrushc_entities::parser::{AssemblerFunctions, Functions};
use thrushc_errors::{CompilationIssue, CompilationIssueCode, CompilationPosition};
use thrushc_logging::LoggingType;
use thrushc_options::{CompilationUnit, CompilerOptions};
use thrushc_span::Span;

use thrushc_token::{Token, traits::TokenExtensions};
use thrushc_token_type::TokenType;

use crate::{
    control::{ParserControlContext, ParserTypeContext},
    table::SymbolsTable,
};

mod attributes;
mod builtins;
mod control;
mod declarations;
mod expected;
mod expressions;
mod impls;
mod modificators;
mod reinterpret;
mod statements;
mod sync;
mod table;
mod traits;
mod typegen;

#[derive(Debug)]
pub struct ParserContext<'parser> {
    tokens: &'parser [Token],
    ast: Vec<Ast<'parser>>,

    errors: Vec<CompilationIssue>,
    bugs: Vec<CompilationIssue>,

    control_ctx: ParserControlContext,
    type_ctx: ParserTypeContext,

    options: &'parser CompilerOptions,

    diagnostician: Diagnostician,
    table: SymbolsTable<'parser>,

    current: usize,
    scope: usize,
}

#[derive(Debug)]
pub struct Parser<'parser> {
    tokens: &'parser [Token],
    file: &'parser CompilationUnit,
}

impl<'parser> Parser<'parser> {
    #[inline]
    pub fn parse(
        tokens: &'parser [Token],
        file: &'parser CompilationUnit,
        options: &'parser CompilerOptions,
    ) -> (ParserContext<'parser>, bool) {
        Self { tokens, file }.start_parsing(options)
    }
}

impl<'parser> Parser<'parser> {
    fn start_parsing(
        &mut self,
        options: &'parser CompilerOptions,
    ) -> (ParserContext<'parser>, bool) {
        let mut ctx: ParserContext = ParserContext::new(self.tokens, self.file, options);

        declarations::parse_forward(&mut ctx);

        while !ctx.is_eof() {
            match declarations::parse(&mut ctx) {
                Ok(ast) => ctx.add_ast(ast),
                Err(error) => {
                    if error.is_bug() {
                        ctx.add_bug(error);
                    } else {
                        ctx.add_error(error);
                    }

                    if let Either::Left(ast) = ctx.sync() {
                        ctx.add_ast(ast);
                    }
                }
            }
        }

        let throwed_errors: bool = ctx.verify();

        (ctx, throwed_errors)
    }
}

impl<'parser> ParserContext<'parser> {
    pub fn new(
        tokens: &'parser [Token],
        file: &'parser CompilationUnit,
        options: &'parser CompilerOptions,
    ) -> Self {
        let functions: Functions = Functions::with_capacity(u8::MAX as usize);
        let asm_functions: AssemblerFunctions = AssemblerFunctions::with_capacity(u8::MAX as usize);

        let control_ctx: ParserControlContext = ParserControlContext::new();

        let table: SymbolsTable =
            SymbolsTable::with_functions(functions, asm_functions, options, file);

        let type_ctx: ParserTypeContext = ParserTypeContext::default();

        Self {
            tokens,
            ast: Vec::with_capacity(u8::MAX as usize),

            errors: Vec::with_capacity(u8::MAX as usize),
            bugs: Vec::with_capacity(u8::MAX as usize),

            control_ctx,
            type_ctx,

            options,

            diagnostician: Diagnostician::new(file, options),
            table,

            current: 0,
            scope: 0,
        }
    }
}

impl<'parser> ParserContext<'parser> {
    pub fn verify(&mut self) -> bool {
        if !self.errors.is_empty() || !self.bugs.is_empty() {
            self.bugs.iter().for_each(|bug: &CompilationIssue| {
                self.diagnostician
                    .dispatch_diagnostic(bug, LoggingType::Bug);
            });

            self.errors.iter().for_each(|error: &CompilationIssue| {
                self.diagnostician
                    .dispatch_diagnostic(error, LoggingType::Error);
            });

            true
        } else {
            false
        }
    }

    pub fn verify_for_only_parse_signature(&mut self) -> bool {
        !self.errors.is_empty() || !self.bugs.is_empty()
    }
}

impl<'parser> ParserContext<'parser> {
    #[must_use]
    pub fn peek(&mut self) -> &'parser Token {
        self.tokens.get(self.current).unwrap_or_else(|| {
            let span: Span = self.previous().get_span();

            thrushc_frontend_abort::abort_compilation(
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
    pub fn previous(&mut self) -> &'parser Token {
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
        }

        self.tokens.get(idx).unwrap_or_else(|| {
            let span: Span = self.peek().get_span();

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

impl<'parser> ParserContext<'parser> {
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

        let next_index: usize = self.current.saturating_add(modifier);

        if next_index >= self.tokens.len() {
            return false;
        }

        self.tokens[next_index].kind == kind
    }

    #[must_use]
    pub fn check_ahead(&mut self, target: TokenType, breakers: &[TokenType]) -> bool {
        let mut last_position: usize = self.current;

        let has_ahead: bool = loop {
            if last_position >= self.tokens.len() {
                break false;
            }

            if breakers.contains(&self.tokens[last_position].kind) {
                break false;
            }

            if self.tokens[last_position].kind == target {
                break true;
            }

            last_position = last_position.saturating_add(1);
        };

        has_ahead
    }
}

impl<'parser> ParserContext<'parser> {
    #[inline]
    pub fn consume(
        &mut self,
        kind: TokenType,
        code: CompilationIssueCode,
        help: String,
    ) -> Result<&'parser Token, CompilationIssue> {
        if self.peek().get_type() == kind {
            return self.advance();
        }

        Err(CompilationIssue::Error(
            code,
            help,
            None,
            self.previous().get_span(),
        ))
    }

    #[inline]
    pub fn consume_these(
        &mut self,
        these: &[TokenType],
        code: CompilationIssueCode,
        help: String,
    ) -> Result<&'parser Token, CompilationIssue> {
        if these.contains(&self.peek().get_type()) {
            return self.advance();
        }

        Err(CompilationIssue::Error(
            code,
            help,
            None,
            self.previous().get_span(),
        ))
    }

    #[inline]
    pub fn go_back(&mut self) {
        self.current = self.current.saturating_sub(1);
    }

    #[inline]
    pub fn match_token(&mut self, kind: TokenType) -> Result<bool, CompilationIssue> {
        if self.peek().kind == kind {
            self.only_advance()?;
            return Ok(true);
        }

        Ok(false)
    }

    #[inline]
    pub fn only_advance(&mut self) -> Result<(), CompilationIssue> {
        if !self.is_eof() {
            self.current = self.current.saturating_add(1);
            Ok(())
        } else {
            Err(CompilationIssue::Error(
                CompilationIssueCode::E0002,
                "EOF has been reached.".into(),
                None,
                self.peek().get_span(),
            ))
        }
    }

    #[inline]
    pub fn advance(&mut self) -> Result<&'parser Token, CompilationIssue> {
        if !self.is_eof() {
            self.current = self.current.saturating_add(1);
            Ok(self.previous())
        } else {
            Err(CompilationIssue::Error(
                CompilationIssueCode::E0002,
                "EOF has been reached.".into(),
                None,
                self.peek().get_span(),
            ))
        }
    }
}

impl<'parser> ParserContext<'parser> {
    pub fn enter_expression(&mut self) -> Result<(), CompilationIssue> {
        let control: &mut ParserControlContext = self.get_mut_control_ctx();

        control.increase_expression_depth();

        const MAX_EXPRESSION_DEPTH: u32 = 516;

        if control.get_expression_depth() > MAX_EXPRESSION_DEPTH {
            let span: Span = self.peek().get_span();

            return Err(CompilationIssue::Error(
                CompilationIssueCode::E0037,
                "Too many depth for a expression. Try to remove some levels of nesting.".into(),
                None,
                span,
            ));
        }

        Ok(())
    }

    pub fn leave_expression(&mut self) {
        self.get_mut_control_ctx().decrease_expression_depth();
    }
}

impl ParserContext<'_> {
    #[inline]
    pub fn reset_scope(&mut self) {
        self.scope = 0;
    }

    #[inline]
    pub fn begin_scope(&mut self) {
        self.scope = self.scope.saturating_add(1);
    }

    #[inline]
    pub fn end_scope(&mut self) {
        self.scope = self.scope.saturating_sub(1);
    }
}

impl<'parser> ParserContext<'parser> {
    #[inline]
    pub fn get_symbols(&self) -> &SymbolsTable<'parser> {
        &self.table
    }

    #[inline]
    pub fn get_control_ctx(&self) -> &ParserControlContext {
        &self.control_ctx
    }

    #[inline]
    pub fn get_type_ctx(&self) -> &ParserTypeContext {
        &self.type_ctx
    }

    #[inline]
    pub fn get_options(&self) -> &CompilerOptions {
        self.options
    }

    #[inline]
    pub fn get_ast(&self) -> &[Ast<'parser>] {
        &self.ast
    }
}

impl<'parser> ParserContext<'parser> {
    #[inline]
    pub fn get_mut_symbols(&mut self) -> &mut SymbolsTable<'parser> {
        &mut self.table
    }

    #[inline]
    pub fn get_mut_control_ctx(&mut self) -> &mut ParserControlContext {
        &mut self.control_ctx
    }

    #[inline]
    pub fn get_mut_type_ctx(&mut self) -> &mut ParserTypeContext {
        &mut self.type_ctx
    }

    #[inline]
    pub fn get_mut_diagnostician(&mut self) -> &mut Diagnostician {
        &mut self.diagnostician
    }
}

impl<'parser> ParserContext<'parser> {
    #[inline]
    pub fn add_ast(&mut self, ast: Ast<'parser>) {
        self.ast.push(ast);
    }

    #[inline]
    pub fn add_error(&mut self, error: CompilationIssue) {
        self.errors.push(error);
    }

    #[inline]
    pub fn add_bug(&mut self, error: CompilationIssue) {
        self.bugs.push(error);
    }
}

impl ParserContext<'_> {
    #[must_use]
    pub fn is_main_scope(&self) -> bool {
        self.scope == 0
    }

    #[inline]
    pub fn get_scope(&self) -> usize {
        self.scope
    }

    #[must_use]
    pub fn is_eof(&mut self) -> bool {
        self.peek().kind == TokenType::Eof
    }
}
