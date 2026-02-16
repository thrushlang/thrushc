use thrustc_ast::Ast;
use thrustc_errors::CompilationIssue;
use thrustc_span::Span;
use thrustc_token_type::TokenType;

use crate::{
    ParserContext,
    expressions::{self, precedences},
};

#[inline]
pub fn index_precedence<'parser>(
    ctx: &mut ParserContext<'parser>,
) -> Result<Ast<'parser>, CompilationIssue> {
    ctx.enter_expression()?;

    let mut expr: Ast = precedences::property::property_precedence(ctx)?;

    while ctx.match_token(TokenType::LBracket)? {
        let span: Span = ctx.previous().span;

        expr = expressions::index::build_index(ctx, expr, span)?;
    }

    ctx.leave_expression();

    Ok(expr)
}
