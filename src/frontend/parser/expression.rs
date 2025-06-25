use crate::{
    core::errors::standard::ThrushCompilerIssue,
    frontend::{
        lexer::tokentype::TokenType, parser::expressions::precedences::or, types::ast::Ast,
        types::parser::stmts::traits::TokenExtensions,
    },
};

use super::{ParserContext, contexts::SyncPosition};

pub fn build_expression<'parser>(
    parser_context: &mut ParserContext<'parser>,
) -> Result<Ast<'parser>, ThrushCompilerIssue> {
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

    let expr: Ast = or::or_precedence(parser_context)?;

    Ok(expr)
}
