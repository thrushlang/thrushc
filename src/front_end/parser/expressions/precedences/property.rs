use crate::core::errors::standard::ThrushCompilerIssue;

use crate::front_end::lexer::tokentype::TokenType;
use crate::front_end::parser::expressions::precedences::unary;
use crate::front_end::parser::{ParserContext, expressions::property};
use crate::front_end::types::ast::Ast;

pub fn property_precedence<'parser>(
    ctx: &mut ParserContext<'parser>,
) -> Result<Ast<'parser>, ThrushCompilerIssue> {
    let mut expr: Ast = unary::unary_precedence(ctx)?;

    if ctx.match_token(TokenType::Dot)? {
        expr = property::build_property(ctx, expr)?;
    }

    Ok(expr)
}
