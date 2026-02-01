use thrushc_ast::{Ast, traits::AstGetType};
use thrushc_errors::CompilationIssue;
use thrushc_span::Span;
use thrushc_token::{Token, traits::TokenExtensions};
use thrushc_token_type::TokenType;
use thrushc_typesystem::{
    Type,
    traits::{PrecedenceTypeExtensions, TypeIsExtensions},
};

use crate::{ParserContext, expressions::precedences};

pub fn term_precedence<'parser>(
    ctx: &mut ParserContext<'parser>,
) -> Result<Ast<'parser>, CompilationIssue> {
    ctx.enter_expression()?;

    let mut expression: Ast = precedences::factor::factor(ctx)?;

    while ctx.match_token(TokenType::Plus)?
        || ctx.match_token(TokenType::Minus)?
        || ctx.match_token(TokenType::Arith)?
        || ctx.match_token(TokenType::LShift)?
        || ctx.match_token(TokenType::RShift)?
        || ctx.match_token(TokenType::Xor)?
        || ctx.match_token(TokenType::Bor)?
        || ctx.match_token(TokenType::BAnd)?
    {
        let operator_tk: &Token = ctx.previous();
        let operator: TokenType = operator_tk.get_type();
        let span: Span = operator_tk.get_span();

        let right: Ast = precedences::factor::factor(ctx)?;

        let left_type: &Type = expression.get_value_type()?;
        let right_type: &Type = right.get_value_type()?;

        let kind: Type = left_type.get_term_precedence_type(right_type, operator);

        expression = Ast::BinaryOp {
            left: expression.clone().into(),
            operator,
            right: right.into(),
            kind,
            span,
        };
    }

    ctx.leave_expression();

    Ok(expression)
}
