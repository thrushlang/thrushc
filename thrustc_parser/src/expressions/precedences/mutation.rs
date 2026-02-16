use thrustc_ast::Ast;
use thrustc_errors::CompilationIssue;
use thrustc_span::Span;
use thrustc_token::traits::TokenExtensions;
use thrustc_token_type::TokenType;
use thrustc_typesystem::Type;

use crate::{
    ParserContext,
    expressions::{self, precedences},
};

pub fn equal_precedence<'parser>(
    ctx: &mut ParserContext<'parser>,
) -> Result<Ast<'parser>, CompilationIssue> {
    ctx.enter_expression()?;

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

    ctx.leave_expression();

    Ok(expression)
}
