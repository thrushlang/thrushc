use crate::core::diagnostic::span::Span;
use crate::core::errors::standard::CompilationIssue;

use crate::front_end::lexer::{token::Token, tokentype::TokenType};
use crate::front_end::parser::{ParserContext, expressions::precedences::term};
use crate::front_end::types::{ast::Ast, parser::stmts::traits::TokenExtensions};
use crate::front_end::typesystem::types::Type;

pub fn cmp_precedence<'parser>(
    ctx: &mut ParserContext<'parser>,
) -> Result<Ast<'parser>, CompilationIssue> {
    let mut expression: Ast = term::term_precedence(ctx)?;

    if ctx.match_token(TokenType::Greater)?
        || ctx.match_token(TokenType::GreaterEq)?
        || ctx.match_token(TokenType::Less)?
        || ctx.match_token(TokenType::LessEq)?
    {
        let operator_tk: &Token = ctx.previous();

        let operator: TokenType = operator_tk.get_type();
        let span: Span = operator_tk.get_span();

        let right: Ast = term::term_precedence(ctx)?;

        expression = Ast::BinaryOp {
            left: expression.into(),
            operator,
            right: right.into(),
            kind: Type::Bool(span),
            span,
        };
    }

    Ok(expression)
}
