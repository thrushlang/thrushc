use crate::core::diagnostic::span::Span;
use crate::core::errors::standard::CompilationIssue;

use crate::front_end::lexer::{token::Token, tokentype::TokenType};
use crate::front_end::parser::{ParserContext, expressions::precedences::cmp};
use crate::front_end::types::{ast::Ast, parser::stmts::traits::TokenExtensions};
use crate::front_end::typesystem::types::Type;

pub fn equality_precedence<'parser>(
    ctx: &mut ParserContext<'parser>,
) -> Result<Ast<'parser>, CompilationIssue> {
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
