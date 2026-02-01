use thrushc_ast::Ast;
use thrushc_errors::CompilationIssue;
use thrushc_span::Span;
use thrushc_token::{Token, traits::TokenExtensions};
use thrushc_token_type::TokenType;
use thrushc_typesystem::Type;

use crate::{ParserContext, expressions::precedences};

pub fn cmp_precedence<'parser>(
    ctx: &mut ParserContext<'parser>,
) -> Result<Ast<'parser>, CompilationIssue> {
    ctx.enter_expression()?;

    let mut expression: Ast = precedences::term::term_precedence(ctx)?;

    if ctx.match_token(TokenType::Greater)?
        || ctx.match_token(TokenType::GreaterEq)?
        || ctx.match_token(TokenType::Less)?
        || ctx.match_token(TokenType::LessEq)?
    {
        let operator_tk: &Token = ctx.previous();

        let operator: TokenType = operator_tk.get_type();
        let span: Span = operator_tk.get_span();

        let right: Ast = precedences::term::term_precedence(ctx)?;

        expression = Ast::BinaryOp {
            left: expression.into(),
            operator,
            right: right.into(),
            kind: Type::Bool(span),
            span,
        };
    }

    ctx.leave_expression();

    Ok(expression)
}
