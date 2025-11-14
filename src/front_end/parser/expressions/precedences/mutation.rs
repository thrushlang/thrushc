use crate::core::errors::standard::ThrushCompilerIssue;

use crate::front_end::lexer::{span::Span, tokentype::TokenType};
use crate::front_end::parser::expressions::precedences::cast;
use crate::front_end::parser::{ParserContext, expr};
use crate::front_end::types::{ast::Ast, parser::stmts::traits::TokenExtensions};
use crate::front_end::typesystem::types::Type;

pub fn equal_precedence<'parser>(
    ctx: &mut ParserContext<'parser>,
) -> Result<Ast<'parser>, ThrushCompilerIssue> {
    let mut expression: Ast = cast::cast_precedence(ctx)?;

    if ctx.match_token(TokenType::Eq)? {
        let span: Span = ctx.previous().get_span();

        let expr: Ast = expr::build_expr(ctx)?;

        expression = Ast::Mut {
            source: expression.into(),
            value: expr.into(),
            kind: Type::Void,
            span,
        };
    }

    Ok(expression)
}
