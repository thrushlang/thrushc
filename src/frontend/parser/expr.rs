use crate::{
    core::errors::standard::ThrushCompilerIssue,
    frontend::{
        lexer::tokentype::TokenType,
        parser::{checks, expressions::precedences::or},
        types::ast::Ast,
    },
};

use super::{ParserContext, contexts::sync::SyncPosition};

pub fn build_expression<'parser>(
    parser_context: &mut ParserContext<'parser>,
) -> Result<Ast<'parser>, ThrushCompilerIssue> {
    self::check_state(parser_context)?;

    parser_context
        .get_mut_control_ctx()
        .set_sync_position(SyncPosition::Expression);

    let expression: Ast = or::or_precedence(parser_context)?;

    parser_context.consume(
        TokenType::SemiColon,
        String::from("Syntax error"),
        String::from("Expected ';'."),
    )?;

    Ok(expression)
}

pub fn build_expr<'parser>(
    parser_context: &mut ParserContext<'parser>,
) -> Result<Ast<'parser>, ThrushCompilerIssue> {
    self::check_state(parser_context)?;

    parser_context
        .get_mut_control_ctx()
        .set_sync_position(SyncPosition::Expression);

    let expr: Ast = or::or_precedence(parser_context)?;

    Ok(expr)
}

fn check_state(parser_context: &mut ParserContext) -> Result<(), ThrushCompilerIssue> {
    checks::check_unreacheable_state(parser_context)
}
