use thrushc_ast::{Ast, traits::AstGetType};
use thrushc_errors::CompilationIssue;
use thrushc_span::Span;
use thrushc_token::{Token, traits::TokenExtensions};
use thrushc_token_type::TokenType;
use thrushc_typesystem::Type;

use crate::{ParserContext, expressions::precedences};

pub fn factor<'parser>(ctx: &mut ParserContext<'parser>) -> Result<Ast<'parser>, CompilationIssue> {
    ctx.enter_expression()?;

    let mut expression: Ast = precedences::mutation::equal_precedence(ctx)?;

    while ctx.match_token(TokenType::Slash)? || ctx.match_token(TokenType::Star)? {
        let operator_tk: &Token = ctx.previous();
        let operator: TokenType = operator_tk.get_type();
        let span: Span = operator_tk.get_span();

        let right: Ast = precedences::mutation::equal_precedence(ctx)?;

        let left_type: &Type = expression.get_value_type()?;

        expression = Ast::BinaryOp {
            left: expression.clone().into(),
            operator,
            right: right.into(),
            kind: left_type.clone(),
            span,
        };
    }

    ctx.leave_expression();

    Ok(expression)
}
