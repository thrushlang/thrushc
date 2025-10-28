use crate::core::errors::standard::ThrushCompilerIssue;

use crate::frontend::lexer::{span::Span, token::Token, tokentype::TokenType};
use crate::frontend::parser::{ParserContext, expressions::precedences::equality};
use crate::frontend::types::ast::Ast;
use crate::frontend::typesystem::types::Type;

pub fn and_precedence<'parser>(
    ctx: &mut ParserContext<'parser>,
) -> Result<Ast<'parser>, ThrushCompilerIssue> {
    let mut expression: Ast = equality::equality_precedence(ctx)?;

    while ctx.match_token(TokenType::And)? {
        let operator_tk: &Token = ctx.previous();

        let operator: TokenType = operator_tk.kind;
        let span: Span = operator_tk.span;

        let right: Ast = equality::equality_precedence(ctx)?;

        expression = Ast::BinaryOp {
            left: expression.into(),
            operator,
            right: right.into(),
            kind: Type::Bool,
            span,
        }
    }

    Ok(expression)
}
