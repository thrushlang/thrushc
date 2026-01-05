use crate::{
    ParserContext,
    expressions::{self, precedences},
};
use thrushc_ast::Ast;
use thrushc_errors::CompilationIssue;
use thrushc_token::tokentype::TokenType;

pub fn property_precedence<'parser>(
    ctx: &mut ParserContext<'parser>,
) -> Result<Ast<'parser>, CompilationIssue> {
    let mut expr: Ast = precedences::unary::unary_precedence(ctx)?;

    if ctx.match_token(TokenType::Dot)? {
        expr = expressions::property::build_property(ctx, expr)?;
    }

    Ok(expr)
}
