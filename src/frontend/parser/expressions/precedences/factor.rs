use crate::{
    core::errors::standard::ThrushCompilerIssue,
    frontend::{
        lexer::{span::Span, token::Token, tokentype::TokenType},
        parser::{ParserContext, expressions::precedences::eq},
        types::{ast::Ast, parser::stmts::traits::TokenExtensions},
        typesystem::types::Type,
    },
};

pub fn factor<'parser>(
    parser_context: &mut ParserContext<'parser>,
) -> Result<Ast<'parser>, ThrushCompilerIssue> {
    let mut expression: Ast = eq::equal_precedence(parser_context)?;

    while parser_context.match_token(TokenType::Slash)?
        || parser_context.match_token(TokenType::Star)?
    {
        let operator_tk: &Token = parser_context.previous();
        let operator: TokenType = operator_tk.get_type();
        let span: Span = operator_tk.get_span();

        let right: Ast = eq::equal_precedence(parser_context)?;

        let left_type: &Type = expression.get_value_type()?;
        let right_type: &Type = right.get_value_type()?;

        let kind: &Type = left_type.precompute_numeric_type(right_type);

        expression = Ast::BinaryOp {
            left: expression.clone().into(),
            operator,
            right: right.into(),
            kind: kind.clone(),
            span,
        };
    }

    Ok(expression)
}
