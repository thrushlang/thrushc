use either::Either;

use crate::front_end::lexer::token::Token;
use crate::front_end::lexer::tokentype::TokenType;
use crate::front_end::parser::ParserContext;
use crate::front_end::parser::contexts::sync::ParserSyncPosition;
use crate::front_end::parser::statements::block;
use crate::front_end::types::ast::Ast;

pub const SYNC_STATEMENTS: [TokenType; 11] = [
    TokenType::Return,
    TokenType::Local,
    TokenType::For,
    TokenType::New,
    TokenType::If,
    TokenType::While,
    TokenType::Continue,
    TokenType::Break,
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
    pub fn sync(&mut self) -> Either<Ast<'parser>, ()> {
        if let Some(position) = self.get_control_ctx().get_sync_position() {
            return match position {
                ParserSyncPosition::Declaration => {
                    self::sync_with_declaration(self);
                    Either::Right(())
                }
                ParserSyncPosition::Statement => self::sync_with_statement(self),
                ParserSyncPosition::Expression => self::sync_with_expression(self),
                ParserSyncPosition::NoRelevant => Either::Right(()),
            };
        }

        Either::Right(())
    }
}

fn sync_with_declaration(ctx: &mut ParserContext) {
    loop {
        if ctx.is_eof() {
            break;
        }

        let peeked: &Token = ctx.peek();

        if SYNC_DECLARATIONS.contains(&peeked.kind) {
            break;
        }

        let _ = ctx.only_advance();
    }

    ctx.get_mut_symbols().finish_parameters();
    ctx.get_mut_symbols().finish_scopes();

    ctx.reset_scope();
}

fn sync_with_statement<'parser>(ctx: &mut ParserContext<'parser>) -> Either<Ast<'parser>, ()> {
    loop {
        if ctx.is_eof() {
            break;
        }

        if ctx.check(TokenType::RBrace) {
            let _ = ctx.only_advance();

            ctx.get_mut_symbols().end_scope();
            ctx.end_scope();

            if ctx.is_main_scope() {
                ctx.get_mut_symbols().finish_parameters();
            }

            break;
        }

        let peeked: &Token = ctx.peek();

        if SYNC_STATEMENTS.contains(&peeked.kind) || SYNC_DECLARATIONS.contains(&peeked.kind) {
            if !ctx.is_main_scope() {
                ctx.get_mut_symbols().end_scope();
                ctx.end_scope();

                if ctx.is_main_scope() {
                    ctx.get_mut_symbols().finish_parameters();
                }

                if let Ok(ast) = block::build_block_without_start(ctx) {
                    return Either::Left(ast);
                } else {
                    return Either::Right(());
                }
            }

            break;
        }

        let _ = ctx.only_advance();
    }

    Either::Right(())
}

fn sync_with_expression<'parser>(ctx: &mut ParserContext<'parser>) -> Either<Ast<'parser>, ()> {
    loop {
        if ctx.is_eof() {
            break;
        }

        if ctx.check(TokenType::RBrace) {
            let _ = ctx.only_advance();

            if !ctx.is_main_scope() {
                ctx.get_mut_symbols().end_scope();
                ctx.end_scope();

                break;
            }

            ctx.get_mut_symbols().finish_parameters();

            break;
        }

        let peeked: &Token = ctx.peek();

        if SYNC_STATEMENTS.contains(&peeked.kind) || SYNC_DECLARATIONS.contains(&peeked.kind) {
            if !ctx.is_main_scope() {
                ctx.get_mut_symbols().end_scope();
                ctx.end_scope();

                if ctx.is_main_scope() {
                    ctx.get_mut_symbols().finish_parameters();
                }

                if let Ok(ast) = block::build_block_without_start(ctx) {
                    return Either::Left(ast);
                } else {
                    return Either::Right(());
                }
            }

            break;
        }

        let _ = ctx.only_advance();
    }

    Either::Right(())
}
