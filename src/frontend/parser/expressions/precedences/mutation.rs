use crate::{
    core::errors::standard::ThrushCompilerIssue,
    frontend::{
        lexer::{span::Span, tokentype::TokenType},
        parser::{ParserContext, expr, expressions::precedences::property},
        types::{ast::Ast, parser::stmts::traits::TokenExtensions},
        typesystem::types::Type,
    },
};

pub fn equal_precedence<'parser>(
    parser_context: &mut ParserContext<'parser>,
) -> Result<Ast<'parser>, ThrushCompilerIssue> {
    let mut expression: Ast = property::property_precedence(parser_context)?;

    if parser_context.match_token(TokenType::Eq)? {
        let span: Span = parser_context.previous().get_span();

        let expr: Ast = expr::build_expr(parser_context)?;

        expression = Ast::Mut {
            source: expression.into(),
            value: expr.into(),
            kind: Type::Void,
            span,
        };
    }

    Ok(expression)
}
