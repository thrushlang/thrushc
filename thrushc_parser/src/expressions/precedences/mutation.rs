use thrushc_ast::Ast;
use thrushc_errors::CompilationIssue;
use thrushc_span::Span;
use thrushc_token::{tokentype::TokenType, traits::TokenExtensions};
use thrushc_typesystem::Type;

use crate::{
    ParserContext,
    expressions::{self, precedences},
};

pub fn equal_precedence<'parser>(
    ctx: &mut ParserContext<'parser>,
) -> Result<Ast<'parser>, CompilationIssue> {
    let mut expression: Ast = precedences::cast::cast_precedence(ctx)?;

    if ctx.match_token(TokenType::Eq)? {
        let span: Span = ctx.previous().get_span();

        let expr: Ast = expressions::build_expr(ctx)?;

        expression = Ast::Mut {
            source: expression.into(),
            value: expr.into(),
            kind: Type::Void(span),
            span,
        };
    }

    Ok(expression)
}
