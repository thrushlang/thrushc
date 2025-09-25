pub mod attributes;
pub mod builtins;
pub mod checks;
pub mod contexts;
pub mod declaration;
pub mod declarations;
pub mod expr;
pub mod expressions;
pub mod parse;
pub mod statement;
pub mod statements;
pub mod symbols;
pub mod sync;
pub mod typegen;

use ahash::AHashMap as HashMap;

use contexts::typectx::ParserTypeContext;
use symbols::SymbolsTable;

use crate::core::compiler::options::CompilationUnit;
use crate::core::console::logging::{self, LoggingType};
use crate::core::diagnostic::diagnostician::Diagnostician;
use crate::core::errors::standard::ThrushCompilerIssue;

use crate::frontends::classical::lexer::token::Token;
use crate::frontends::classical::lexer::tokentype::TokenType;
use crate::frontends::classical::parser::contexts::controlctx::ParserControlContext;
use crate::frontends::classical::types::ast::Ast;
use crate::frontends::classical::types::parser::symbols::types::{AssemblerFunctions, Functions};

const MINIMAL_AST_CAPACITY: usize = 100_000;
const MINIMAL_GLOBAL_CAPACITY: usize = 2024;
const MAX_ERRORS: usize = 50;

#[derive(Debug)]
pub struct ParserContext<'parser> {
    tokens: &'parser [Token],
    ast: Vec<Ast<'parser>>,

    errors: Vec<ThrushCompilerIssue>,
    bugs: Vec<ThrushCompilerIssue>,

    control_ctx: ParserControlContext,
    type_ctx: ParserTypeContext,

    diagnostician: Diagnostician,
    symbols: SymbolsTable<'parser>,

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
    ) -> (ParserContext<'parser>, bool) {
        Self { tokens, file }.start()
    }
}

impl<'parser> Parser<'parser> {
    fn start(&mut self) -> (ParserContext<'parser>, bool) {
        let mut parser_context: ParserContext = ParserContext::new(self.tokens, self.file);

        declaration::parse_forward(&mut parser_context);

        while !parser_context.is_eof() {
            match declaration::decl(&mut parser_context) {
                Ok(ast) => {
                    parser_context.add_ast(ast);
                }

                Err(error) => {
                    let total_issues: usize =
                        parser_context.errors.len() + parser_context.bugs.len();

                    if total_issues >= MAX_ERRORS {
                        logging::print_warn(
                            LoggingType::Warning,
                            "Too many issues. Stopping compilation.",
                        );

                        break;
                    }

                    if error.is_bug() {
                        parser_context.add_bug(error);
                    } else {
                        parser_context.add_error(error);
                    }

                    if let Err(error) = parser_context.sync() {
                        parser_context.add_error(error);
                    }
                }
            }
        }

        let throwed_errors: bool = parser_context.verify();

        (parser_context, throwed_errors)
    }
}

impl<'parser> ParserContext<'parser> {
    pub fn new(tokens: &'parser [Token], file: &'parser CompilationUnit) -> Self {
        let functions: Functions = HashMap::with_capacity(MINIMAL_GLOBAL_CAPACITY);
        let asm_functions: AssemblerFunctions = HashMap::with_capacity(MINIMAL_GLOBAL_CAPACITY);

        Self {
            tokens,
            ast: Vec::with_capacity(MINIMAL_AST_CAPACITY),

            errors: Vec::with_capacity(100),
            bugs: Vec::with_capacity(100),

            control_ctx: ParserControlContext::new(),
            type_ctx: ParserTypeContext::new(),

            diagnostician: Diagnostician::new(file),
            symbols: SymbolsTable::with_functions(functions, asm_functions),

            current: 0,
            scope: 0,
        }
    }

    pub fn verify(&mut self) -> bool {
        if !self.errors.is_empty() || !self.bugs.is_empty() {
            self.bugs.iter().for_each(|bug: &ThrushCompilerIssue| {
                self.diagnostician.build_diagnostic(bug, LoggingType::Bug);
            });

            self.errors.iter().for_each(|error: &ThrushCompilerIssue| {
                self.diagnostician
                    .build_diagnostic(error, LoggingType::Error);
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
    pub fn is_unreacheable_code(&self) -> bool {
        self.control_ctx.get_unreacheable_code_scope() == self.scope && !self.is_main_scope()
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

    #[inline]
    pub fn match_token(&mut self, kind: TokenType) -> Result<bool, ThrushCompilerIssue> {
        if self.peek().kind == kind {
            self.only_advance()?;
            return Ok(true);
        }

        Ok(false)
    }

    #[inline]
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

    #[inline]
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
}

impl<'parser> ParserContext<'parser> {
    #[inline]
    pub fn get_symbols(&self) -> &SymbolsTable<'parser> {
        &self.symbols
    }

    #[inline]
    pub fn get_control_ctx(&mut self) -> &ParserControlContext {
        &self.control_ctx
    }

    #[inline]
    pub fn get_type_ctx(&self) -> &ParserTypeContext {
        &self.type_ctx
    }

    #[inline]
    pub fn get_scope(&self) -> usize {
        self.scope
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
    pub fn get_mut_type_ctx(&mut self) -> &mut ParserTypeContext {
        &mut self.type_ctx
    }

    #[inline]
    pub fn get_mut_control_ctx(&mut self) -> &mut ParserControlContext {
        &mut self.control_ctx
    }

    #[inline]
    pub fn get_mut_scope(&mut self) -> &mut usize {
        &mut self.scope
    }
}

impl<'parser> ParserContext<'parser> {
    #[inline]
    pub fn add_ast(&mut self, ast: Ast<'parser>) {
        self.ast.push(ast);
    }

    #[inline]
    pub fn add_error(&mut self, error: ThrushCompilerIssue) {
        self.errors.push(error);
    }

    #[inline]
    pub fn add_bug(&mut self, error: ThrushCompilerIssue) {
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
