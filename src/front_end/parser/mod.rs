pub mod attributes;
pub mod builder;
pub mod builtins;
pub mod constants;
pub mod contexts;
pub mod declarations;
pub mod expressions;
pub mod interpret;
pub mod statements;
pub mod symbols;
pub mod sync;
pub mod typegen;

use crate::core::compiler::options::{CompilationUnit, CompilerOptions};
use crate::core::console::logging::{self, LoggingType};
use crate::core::diagnostic::diagnostician::Diagnostician;
use crate::core::errors::standard::{CompilationIssue, CompilationIssueCode};

use crate::front_end::lexer::token::Token;
use crate::front_end::lexer::tokentype::TokenType;
use crate::front_end::parser::contexts::controlctx::ParserControlContext;
use crate::front_end::parser::contexts::typectx::ParserTypeContext;
use crate::front_end::parser::symbols::SymbolsTable;
use crate::front_end::preprocessor::module::Module;
use crate::front_end::types::ast::Ast;
use crate::front_end::types::parser::symbols::types::{AssemblerFunctions, Functions};

use ahash::AHashMap as HashMap;

pub const PARSER_MINIMAL_AST_CAPACITY: usize = 10_000;
pub const PARSER_MINIMAL_GLOBAL_CAPACITY: usize = 10_000;

#[derive(Debug)]
pub struct ParserContext<'parser> {
    tokens: &'parser [Token],
    ast: Vec<Ast<'parser>>,

    errors: Vec<CompilationIssue>,
    bugs: Vec<CompilationIssue>,

    control_ctx: ParserControlContext,
    type_ctx: ParserTypeContext,

    diagnostician: Diagnostician,
    symbols: SymbolsTable<'parser>,

    current: usize,
    scope: usize,

    abort: bool,
}

#[derive(Debug)]
pub struct Parser<'parser> {
    tokens: &'parser [Token],
    file: &'parser CompilationUnit,
    modules: Vec<Module>,
}

impl<'parser> Parser<'parser> {
    #[inline]
    pub fn parse(
        tokens: &'parser [Token],
        file: &'parser CompilationUnit,
        modules: Vec<Module>,
        options: &CompilerOptions,
    ) -> (ParserContext<'parser>, bool) {
        Self {
            tokens,
            file,
            modules,
        }
        .start(options)
    }
}

impl<'parser> Parser<'parser> {
    fn start(&mut self, options: &CompilerOptions) -> (ParserContext<'parser>, bool) {
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

                    ctx.sync();

                    if ctx.need_abort() {
                        break;
                    }
                }
            }
        }

        let abort: bool = ctx.need_abort();
        let throwed_errors: bool = ctx.verify();

        (ctx, throwed_errors || abort)
    }
}

impl<'parser> ParserContext<'parser> {
    pub fn new(
        tokens: &'parser [Token],
        file: &'parser CompilationUnit,
        options: &CompilerOptions,
    ) -> Self {
        let functions: Functions = HashMap::with_capacity(PARSER_MINIMAL_GLOBAL_CAPACITY);
        let asm_functions: AssemblerFunctions =
            HashMap::with_capacity(PARSER_MINIMAL_GLOBAL_CAPACITY);

        Self {
            tokens,
            ast: Vec::with_capacity(PARSER_MINIMAL_AST_CAPACITY),

            errors: Vec::with_capacity(100),
            bugs: Vec::with_capacity(100),

            control_ctx: ParserControlContext::new(),
            type_ctx: ParserTypeContext::new(),

            diagnostician: Diagnostician::new(file, options),
            symbols: SymbolsTable::with_functions(functions, asm_functions),

            current: 0,
            scope: 0,

            abort: false,
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

            return true;
        }

        false
    }
}

impl<'parser> ParserContext<'parser> {
    #[must_use]
    pub fn peek(&self) -> &'parser Token {
        self.tokens.get(self.current).unwrap_or_else(|| {
            logging::print_frontend_panic(
                LoggingType::FrontEndPanic,
                "Attempting to get token in invalid current position.",
            );
        })
    }

    #[must_use]
    pub fn previous(&self) -> &'parser Token {
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

impl<'parser> ParserContext<'parser> {
    #[inline]
    pub fn need_abort(&self) -> bool {
        self.abort
    }
}

impl<'parser> ParserContext<'parser> {
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

impl<'parser> ParserContext<'parser> {
    #[inline]
    pub fn consume(
        &mut self,
        kind: TokenType,
        code: CompilationIssueCode,
        help: String,
    ) -> Result<&'parser Token, CompilationIssue> {
        if self.peek().kind == kind {
            return self.advance();
        }

        Err(CompilationIssue::Error(
            code,
            help,
            None,
            self.previous().span,
        ))
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
            self.current += 1;
            return Ok(());
        }

        Err(CompilationIssue::Error(
            CompilationIssueCode::E0002,
            String::from("EOF has been reached."),
            None,
            self.peek().span,
        ))
    }

    #[inline]
    pub fn advance(&mut self) -> Result<&'parser Token, CompilationIssue> {
        if !self.is_eof() {
            self.current += 1;
            return Ok(self.previous());
        }

        Err(CompilationIssue::Error(
            CompilationIssueCode::E0002,
            String::from("EOF has been reached."),
            None,
            self.peek().span,
        ))
    }

    #[inline]
    pub fn set_force_abort(&mut self) {
        self.abort = true;
    }
}

impl ParserContext<'_> {
    #[inline]
    pub fn reset_scope(&mut self) {
        self.scope = 0;
    }

    #[inline]
    pub fn begin_scope(&mut self) {
        self.scope += 1;
    }

    #[inline]
    pub fn end_scope(&mut self) {
        self.scope -= 1;
    }
}

impl<'parser> ParserContext<'parser> {
    #[inline]
    pub fn get_symbols(&self) -> &SymbolsTable<'parser> {
        &self.symbols
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
    pub fn get_ast(&self) -> &[Ast<'parser>] {
        &self.ast
    }
}

impl<'parser> ParserContext<'parser> {
    #[inline]
    pub fn get_mut_symbols(&mut self) -> &mut SymbolsTable<'parser> {
        &mut self.symbols
    }

    #[inline]
    pub fn get_mut_control_ctx(&mut self) -> &mut ParserControlContext {
        &mut self.control_ctx
    }

    #[inline]
    pub fn get_mut_type_ctx(&mut self) -> &mut ParserTypeContext {
        &mut self.type_ctx
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

    #[must_use]
    pub fn is_eof(&self) -> bool {
        self.peek().kind == TokenType::Eof
    }
}
