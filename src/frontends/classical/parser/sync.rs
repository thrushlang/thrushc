use crate::{
    core::errors::standard::ThrushCompilerIssue,
    frontends::classical::{
        lexer::{token::Token, tokentype::TokenType},
        parser::{ParserContext, contexts::sync::ParserSyncPosition},
    },
};

pub const SYNC_STATEMENTS: [TokenType; 9] = [
    TokenType::Return,
    TokenType::Local,
    TokenType::For,
    TokenType::New,
    TokenType::If,
    TokenType::While,
    TokenType::Continue,
    TokenType::Break,
    TokenType::Loop,
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
    pub fn sync(&mut self) -> Result<(), ThrushCompilerIssue> {
        match self.control_ctx.get_sync_position() {
            ParserSyncPosition::Declaration => self::sync_with_declaration(self)?,
            ParserSyncPosition::Statement => self::sync_with_statement(self)?,
            ParserSyncPosition::Expression => self::sync_with_expression(self)?,

            ParserSyncPosition::NoRelevant => (),
        }

        self.control_ctx
            .set_sync_position(ParserSyncPosition::NoRelevant);

        Ok(())
    }
}

fn sync_with_declaration(ctx: &mut ParserContext) -> Result<(), ThrushCompilerIssue> {
    loop {
        if ctx.is_eof() {
            break;
        }

        let peeked: &Token = ctx.peek();

        if SYNC_DECLARATIONS.contains(&peeked.kind) {
            break;
        }

        ctx.only_advance()?;
    }

    ctx.scope = 0;
    ctx.symbols.finish_parameters();

    Ok(())
}

fn sync_with_statement(ctx: &mut ParserContext) -> Result<(), ThrushCompilerIssue> {
    loop {
        if ctx.is_eof() {
            break;
        }

        if ctx.check(TokenType::RBrace) {
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

        ctx.only_advance()?;
    }

    Ok(())
}

fn sync_with_expression(ctx: &mut ParserContext) -> Result<(), ThrushCompilerIssue> {
    loop {
        if ctx.is_eof() {
            break;
        }

        if ctx.check(TokenType::RBrace) {
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

        ctx.only_advance()?;
    }

    Ok(())
}
