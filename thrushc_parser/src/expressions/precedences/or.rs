use thrushc_ast::Ast;
use thrushc_errors::CompilationIssue;
use thrushc_span::Span;
use thrushc_token::{Token, tokentype::TokenType, traits::TokenExtensions};
use thrushc_typesystem::Type;

use crate::{ParserContext, expressions::precedences};

pub fn or_precedence<'parser>(
    ctx: &mut ParserContext<'parser>,
) -> Result<Ast<'parser>, CompilationIssue> {
    let mut expression: Ast = precedences::and::and_precedence(ctx)?;

    while ctx.match_token(TokenType::Or)? {
        let operator_tk: &Token = ctx.previous();

        let operator: TokenType = operator_tk.kind;
        let span: Span = operator_tk.get_span();

        let right: Ast = precedences::and::and_precedence(ctx)?;

        expression = Ast::BinaryOp {
            left: expression.into(),
            operator,
            right: right.into(),
            kind: Type::Bool(span),
            span,
        }
    }

    Ok(expression)
}
