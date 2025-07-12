use crate::{
    core::errors::standard::ThrushCompilerIssue,
    frontend::{
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

fn sync_with_declaration(parser_context: &mut ParserContext) -> Result<(), ThrushCompilerIssue> {
    loop {
        if parser_context.is_eof() {
            break;
        }

        let peeked: &Token = parser_context.peek();

        if SYNC_DECLARATIONS.contains(&peeked.kind) {
            break;
        }

        parser_context.only_advance()?;
    }

    parser_context.scope = 0;
    parser_context.symbols.end_parameters();

    Ok(())
}

fn sync_with_statement(parser_context: &mut ParserContext) -> Result<(), ThrushCompilerIssue> {
    loop {
        if parser_context.is_eof() {
            break;
        }

        if parser_context.check(TokenType::RBrace) {
            break;
        }

        if parser_context.check(TokenType::SemiColon) {
            let _ = parser_context.only_advance();
            break;
        }

        let peeked: &Token = parser_context.peek();

        if SYNC_STATEMENTS.contains(&peeked.kind) || SYNC_DECLARATIONS.contains(&peeked.kind) {
            break;
        }

        parser_context.only_advance()?;
    }

    Ok(())
}

fn sync_with_expression(parser_context: &mut ParserContext) -> Result<(), ThrushCompilerIssue> {
    loop {
        if parser_context.is_eof() {
            break;
        }

        if parser_context.check(TokenType::RBrace) {
            break;
        }

        if parser_context.check(TokenType::SemiColon) {
            let _ = parser_context.only_advance();
            break;
        }

        let peeked: &Token = parser_context.peek();

        if SYNC_STATEMENTS.contains(&peeked.kind) || SYNC_DECLARATIONS.contains(&peeked.kind) {
            break;
        }

        parser_context.only_advance()?;
    }

    Ok(())
}
