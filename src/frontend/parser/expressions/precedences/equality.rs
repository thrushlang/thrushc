use crate::core::errors::standard::ThrushCompilerIssue;

use crate::frontend::lexer::{span::Span, token::Token, tokentype::TokenType};
use crate::frontend::parser::{ParserContext, expressions::precedences::cmp};
use crate::frontend::types::{ast::Ast, parser::stmts::traits::TokenExtensions};
use crate::frontend::typesystem::types::Type;

pub fn equality_precedence<'parser>(
    ctx: &mut ParserContext<'parser>,
) -> Result<Ast<'parser>, ThrushCompilerIssue> {
    let mut expression: Ast = cmp::cmp_precedence(ctx)?;

    if ctx.match_token(TokenType::BangEq)? || ctx.match_token(TokenType::EqEq)? {
        let operator_tk: &Token = ctx.previous();
        let operator: TokenType = operator_tk.kind;
        let span: Span = operator_tk.get_span();

        let right: Ast = cmp::cmp_precedence(ctx)?;

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
