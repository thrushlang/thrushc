use crate::core::errors::standard::ThrushCompilerIssue;

use crate::front_end::lexer::tokentype::TokenType;
use crate::front_end::parser::checks;
use crate::front_end::parser::expressions::precedences::or;
use crate::front_end::types::ast::Ast;

use super::{ParserContext, contexts::sync::ParserSyncPosition};

pub fn build_expression<'parser>(
    ctx: &mut ParserContext<'parser>,
) -> Result<Ast<'parser>, ThrushCompilerIssue> {
    self::check_state(ctx)?;

    ctx.get_mut_control_ctx()
        .add_sync_position(ParserSyncPosition::Expression);

    let expression: Ast = or::or_precedence(ctx)?;

    ctx.consume(
        TokenType::SemiColon,
        String::from("Syntax error"),
        String::from("Expected ';'."),
    )?;

    ctx.get_mut_control_ctx().pop_sync_position();

    Ok(expression)
}

pub fn build_expr<'parser>(
    ctx: &mut ParserContext<'parser>,
) -> Result<Ast<'parser>, ThrushCompilerIssue> {
    self::check_state(ctx)?;

    ctx.get_mut_control_ctx()
        .add_sync_position(ParserSyncPosition::Expression);

    let expr: Ast = or::or_precedence(ctx)?;

    ctx.get_mut_control_ctx().pop_sync_position();

    Ok(expr)
}

fn check_state(ctx: &mut ParserContext) -> Result<(), ThrushCompilerIssue> {
    checks::check_unreacheable_state(ctx)
}
