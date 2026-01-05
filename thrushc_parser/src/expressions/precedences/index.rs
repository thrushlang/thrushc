use thrushc_ast::Ast;
use thrushc_errors::CompilationIssue;
use thrushc_span::Span;
use thrushc_token::tokentype::TokenType;

use crate::{
    ParserContext,
    expressions::{self, precedences},
};

#[inline]
pub fn index_precedence<'parser>(
    ctx: &mut ParserContext<'parser>,
) -> Result<Ast<'parser>, CompilationIssue> {
    let mut expr: Ast = precedences::property::property_precedence(ctx)?;

    while ctx.match_token(TokenType::LBracket)? {
        let span: Span = ctx.previous().span;

        expr = expressions::index::build_index(ctx, expr, span)?;
    }

    Ok(expr)
}
