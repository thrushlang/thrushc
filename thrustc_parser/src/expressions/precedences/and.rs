use thrustc_ast::Ast;
use thrustc_errors::CompilationIssue;
use thrustc_span::Span;
use thrustc_token::Token;
use thrustc_token_type::TokenType;
use thrustc_typesystem::Type;

use crate::{ParserContext, expressions::precedences};

pub fn and_precedence<'parser>(
    ctx: &mut ParserContext<'parser>,
) -> Result<Ast<'parser>, CompilationIssue> {
    ctx.enter_expression()?;

    let mut expression: Ast = precedences::equality::equality_precedence(ctx)?;

    while ctx.match_token(TokenType::And)? {
        let operator_tk: &Token = ctx.previous();

        let operator: TokenType = operator_tk.kind;
        let span: Span = operator_tk.span;

        let right: Ast = precedences::equality::equality_precedence(ctx)?;

        expression = Ast::BinaryOp {
            left: expression.into(),
            operator,
            right: right.into(),
            kind: Type::Bool(span),
            span,
        }
    }

    ctx.leave_expression();

    Ok(expression)
}
