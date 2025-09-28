use crate::{
    core::errors::standard::ThrushCompilerIssue,
    frontends::classical::{
        lexer::{span::Span, token::Token, tokentype::TokenType},
        parser::{ParserContext, expressions::precedences::factor},
        types::{ast::Ast, parser::stmts::traits::TokenExtensions},
        typesystem::{traits::CastTypeExtensions, types::Type},
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
        || parser_context.match_token(TokenType::Xor)?
        || parser_context.match_token(TokenType::Bor)?
    {
        let operator_tk: &Token = parser_context.previous();
        let operator: TokenType = operator_tk.get_type();
        let span: Span = operator_tk.get_span();

        let right: Ast = factor::factor(parser_context)?;

        let left_type: &Type = expression.get_value_type()?;
        let right_type: &Type = right.get_value_type()?;

        let kind: Type = left_type.precompute(right_type);

        expression = Ast::BinaryOp {
            left: expression.clone().into(),
            operator,
            right: right.into(),
            kind,
            span,
        };
    }

    Ok(expression)
}
