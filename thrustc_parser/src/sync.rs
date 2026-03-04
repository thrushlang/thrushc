use thrustc_ast::Ast;
use thrustc_token::{Token, traits::TokenExtensions};
use thrustc_token_type::TokenType;

use crate::{ParserContext, control::ParserSyncPosition, statements::block};

pub const SYNC_STATEMENTS: [TokenType; 13] = [
    TokenType::Return,
    TokenType::Local,
    TokenType::For,
    TokenType::New,
    TokenType::If,
    TokenType::While,
    TokenType::Continue,
    TokenType::ContinueAll,
    TokenType::Break,
    TokenType::BreakAll,
    TokenType::Loop,
    TokenType::Const,
    TokenType::Static,
];

pub const SYNC_DECLARATIONS: [TokenType; 6] = [
    TokenType::Type,
    TokenType::Struct,
    TokenType::Fn,
    TokenType::Enum,
    TokenType::Const,
    TokenType::Static,
];

impl<'parser> ParserContext<'parser> {
    pub fn sync(&mut self) {
        if let Some(position) = self.get_control_ctx().get_sync_position() {
            match position {
                ParserSyncPosition::Declaration => self::sync_with_declaration(self),
                ParserSyncPosition::Statement => self::sync_with_statement(self),
                ParserSyncPosition::Expression => self::sync_with_expression(self),
                ParserSyncPosition::NoRelevant => (),
            }
        }
    }
}

fn sync_with_declaration(ctx: &mut ParserContext) {
    loop {
        if ctx.is_eof() {
            break;
        }

        let peeked: &Token = ctx.peek();

        if SYNC_DECLARATIONS.contains(&peeked.kind) {
            ctx.get_mut_symbols().finish_parameters();
            ctx.get_mut_symbols().finish_scopes();
            ctx.reset_scope();

            break;
        } else {
            ctx.get_mut_symbols().end_scope();
            ctx.end_scope();
        }

        let _ = ctx.only_advance();
    }
}

fn sync_with_statement<'parser>(ctx: &mut ParserContext<'parser>) {
    loop {
        if ctx.is_eof() {
            break;
        }

        if ctx.is_main_scope() {
            let peeked: &Token = ctx.peek();

            while !SYNC_DECLARATIONS.contains(&peeked.kind) {
                let _ = ctx.only_advance();
            }

            ctx.get_mut_symbols().finish_parameters();
            ctx.get_mut_symbols().finish_scopes();
            ctx.reset_scope();

            break;
        }

        if ctx.check(TokenType::RBrace) {
            let _ = ctx.only_advance();

            if !ctx.is_main_scope() {
                ctx.get_mut_symbols().end_scope();
                ctx.end_scope();
            }

            break;
        }

        let peeked: &Token = ctx.peek();

        if SYNC_STATEMENTS.contains(&peeked.get_type()) {
            let first_: Result<Ast<'_>, thrustc_errors::CompilationIssue> =
                block::build_block_without_start(ctx);

            if first_.is_ok() {
                let mut continue_first_loop: bool = false;

                while ctx.check_ahead(TokenType::RBrace, &SYNC_DECLARATIONS) {
                    let second_: Result<Ast<'_>, thrustc_errors::CompilationIssue> =
                        block::build_block_without_start(ctx);

                    if second_.is_err() {
                        continue_first_loop = true;
                        break;
                    }
                }

                if continue_first_loop {
                    let _ = ctx.only_advance();
                    continue;
                }

                break;
            } else {
                let _ = ctx.only_advance();
                continue;
            }
        }

        let _ = ctx.only_advance();
    }
}

fn sync_with_expression<'parser>(ctx: &mut ParserContext<'parser>) {
    loop {
        if ctx.is_eof() {
            break;
        }

        if ctx.is_main_scope() {
            let peeked: &Token = ctx.peek();

            while !SYNC_DECLARATIONS.contains(&peeked.kind) {
                let _ = ctx.only_advance();
            }

            ctx.get_mut_symbols().finish_parameters();
            ctx.get_mut_symbols().finish_scopes();
            ctx.reset_scope();

            break;
        }

        if ctx.check(TokenType::RBrace) {
            let _ = ctx.only_advance();

            if !ctx.is_main_scope() {
                ctx.get_mut_symbols().end_scope();
                ctx.end_scope();
            }

            break;
        }

        let peeked: &Token = ctx.peek();

        if SYNC_STATEMENTS.contains(&peeked.get_type()) {
            let first_: Result<Ast<'_>, thrustc_errors::CompilationIssue> =
                block::build_block_without_start(ctx);

            if first_.is_ok() {
                let mut continue_first_loop: bool = false;

                while ctx.check_ahead(TokenType::RBrace, &SYNC_DECLARATIONS) {
                    let second_: Result<Ast<'_>, thrustc_errors::CompilationIssue> =
                        block::build_block_without_start(ctx);

                    if second_.is_err() {
                        continue_first_loop = true;
                        break;
                    }
                }

                if continue_first_loop {
                    let _ = ctx.only_advance();
                    continue;
                }

                break;
            } else {
                let _ = ctx.only_advance();
                continue;
            }
        }

        let _ = ctx.only_advance();
    }
}
