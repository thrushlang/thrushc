use crate::{
    core::errors::standard::ThrushCompilerIssue,
    frontend::{
        lexer::{span::Span, token::Token, tokentype::TokenType},
        parser::expressions::{precedences::or, reference},
        types::{
            lexer::ThrushType,
            parser::stmts::{stmt::ThrushStatement, traits::TokenExtensions},
        },
    },
};

use super::{ParserContext, contexts::SyncPosition, typegen};

pub fn build_expression<'instr>(
    parser_context: &mut ParserContext<'instr>,
) -> Result<ThrushStatement<'instr>, ThrushCompilerIssue> {
    parser_context
        .get_mut_control_ctx()
        .set_sync_position(SyncPosition::Expression);

    if parser_context.is_unreacheable_code() {
        return Err(ThrushCompilerIssue::Error(
            String::from("Syntax error"),
            String::from("Unreacheable code."),
            None,
            parser_context.peek().get_span(),
        ));
    }

    let expression: ThrushStatement = or::or_precedence(parser_context)?;

    parser_context.consume(
        TokenType::SemiColon,
        String::from("Syntax error"),
        String::from("Expected ';'."),
    )?;

    Ok(expression)
}

pub fn build_expr<'instr>(
    parser_context: &mut ParserContext<'instr>,
) -> Result<ThrushStatement<'instr>, ThrushCompilerIssue> {
    parser_context
        .get_mut_control_ctx()
        .set_sync_position(SyncPosition::Expression);

    if parser_context.is_unreacheable_code() {
        return Err(ThrushCompilerIssue::Error(
            String::from("Syntax error"),
            String::from("Unreacheable code."),
            None,
            parser_context.peek().get_span(),
        ));
    }

    let expr: ThrushStatement = or::or_precedence(parser_context)?;

    Ok(expr)
}
