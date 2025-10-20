use crate::{
    core::errors::standard::ThrushCompilerIssue,
    frontend::{
        lexer::{span::Span, tokentype::TokenType},
        parser::{ParserContext, expr, expressions::precedences::index},
        types::{ast::Ast, parser::stmts::traits::TokenExtensions},
        typesystem::types::Type,
    },
};

pub fn equal_precedence<'parser>(
    ctx: &mut ParserContext<'parser>,
) -> Result<Ast<'parser>, ThrushCompilerIssue> {
    let mut expression: Ast = index::index_precedence(ctx)?;

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
