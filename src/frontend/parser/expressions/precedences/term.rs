use crate::{
    core::errors::standard::ThrushCompilerIssue,
    frontend::{
        lexer::{span::Span, token::Token, tokentype::TokenType},
        parser::{ParserContext, expressions::precedences::factor},
        types::{ast::Ast, lexer::ThrushType, parser::stmts::traits::TokenExtensions},
    },
};

pub fn term_precedence<'parser>(
    parser_context: &mut ParserContext<'parser>,
) -> Result<Ast<'parser>, ThrushCompilerIssue> {
    let mut expression: Ast = factor::factor(parser_context)?;

    while parser_context.match_token(TokenType::Plus)?
        || parser_context.match_token(TokenType::Minus)?
        || parser_context.match_token(TokenType::LShift)?
        || parser_context.match_token(TokenType::RShift)?
    {
        let operator_tk: &Token = parser_context.previous();
        let operator: TokenType = operator_tk.get_type();
        let span: Span = operator_tk.get_span();

        let right: Ast = factor::factor(parser_context)?;

        let left_type: &ThrushType = expression.get_value_type()?;
        let right_type: &ThrushType = right.get_value_type()?;

        let kind: &ThrushType = left_type.precompute_numeric_type(right_type);

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
