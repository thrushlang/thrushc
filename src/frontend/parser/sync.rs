use crate::frontend::lexer::token::Token;
use crate::frontend::lexer::tokentype::TokenType;
use crate::frontend::parser::ParserContext;
use crate::frontend::parser::contexts::sync::ParserSyncPosition;

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

impl ParserContext<'_> {
    pub fn sync(&mut self) {
        if let Some(position) = self.control_ctx.get_sync_position() {
            match position {
                ParserSyncPosition::Declaration => self::sync_with_declaration(self),
                ParserSyncPosition::Statement => self::sync_with_statement(self),
                ParserSyncPosition::Expression => self::sync_with_expression(self),

                ParserSyncPosition::NoRelevant => (),
            }

            self.control_ctx.pop_sync_position();
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
            break;
        }

        let _ = ctx.only_advance();
    }

    ctx.get_mut_control_ctx().set_inside_function(false);
    ctx.get_mut_symbols().finish_parameters();
    ctx.get_mut_symbols().finish_scopes();

    ctx.reset_scope();
}

fn sync_with_statement(ctx: &mut ParserContext) {
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

        if ctx.check(TokenType::SemiColon) {
            let _ = ctx.only_advance();
            break;
        }

        let peeked: &Token = ctx.peek();

        if SYNC_STATEMENTS.contains(&peeked.kind) || SYNC_DECLARATIONS.contains(&peeked.kind) {
            break;
        }

        let _ = ctx.only_advance();
    }
}

fn sync_with_expression(ctx: &mut ParserContext) {
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

        if ctx.check(TokenType::SemiColon) {
            let _ = ctx.only_advance();
            break;
        }

        let peeked: &Token = ctx.peek();

        if SYNC_STATEMENTS.contains(&peeked.kind) || SYNC_DECLARATIONS.contains(&peeked.kind) {
            break;
        }

        let _ = ctx.only_advance();
    }
}
