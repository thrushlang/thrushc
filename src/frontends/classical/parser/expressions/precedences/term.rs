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
    ctx: &mut ParserContext<'parser>,
) -> Result<Ast<'parser>, ThrushCompilerIssue> {
    let mut expression: Ast = factor::factor(ctx)?;

    while ctx.match_token(TokenType::Plus)?
        || ctx.match_token(TokenType::Minus)?
        || ctx.match_token(TokenType::LShift)?
        || ctx.match_token(TokenType::RShift)?
        || ctx.match_token(TokenType::Xor)?
        || ctx.match_token(TokenType::Bor)?
    {
        let operator_tk: &Token = ctx.previous();
        let operator: TokenType = operator_tk.get_type();
        let span: Span = operator_tk.get_span();

        let right: Ast = factor::factor(ctx)?;

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
