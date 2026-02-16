use crate::{
    ParserContext,
    expressions::{self, precedences},
};
use thrustc_ast::Ast;
use thrustc_errors::CompilationIssue;
use thrustc_token_type::TokenType;

pub fn property_precedence<'parser>(
    ctx: &mut ParserContext<'parser>,
) -> Result<Ast<'parser>, CompilationIssue> {
    ctx.enter_expression()?;

    let mut expr: Ast = precedences::unary::unary_precedence(ctx)?;

    if ctx.match_token(TokenType::Dot)? {
        expr = expressions::property::build_property(ctx, expr)?;
    }

    ctx.leave_expression();

    Ok(expr)
}
