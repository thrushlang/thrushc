use crate::core::diagnostic::span::Span;
use crate::core::errors::standard::CompilationIssue;

use crate::front_end::lexer::tokentype::TokenType;
use crate::front_end::parser::expressions::precedences::cast;
use crate::front_end::parser::{ParserContext, expressions};
use crate::front_end::types::{ast::Ast, parser::stmts::traits::TokenExtensions};
use crate::front_end::typesystem::types::Type;

pub fn equal_precedence<'parser>(
    ctx: &mut ParserContext<'parser>,
) -> Result<Ast<'parser>, CompilationIssue> {
    let mut expression: Ast = cast::cast_precedence(ctx)?;

    if ctx.match_token(TokenType::Eq)? {
        let span: Span = ctx.previous().get_span();

        let expr: Ast = expressions::build_expr(ctx)?;

        expression = Ast::Mut {
            source: expression.into(),
            value: expr.into(),
            kind: Type::Void,
            span,
        };
    }

    Ok(expression)
}
