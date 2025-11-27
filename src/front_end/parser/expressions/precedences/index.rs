use crate::core::errors::standard::CompilationIssue;

use crate::front_end::lexer::{span::Span, tokentype::TokenType};
use crate::front_end::parser::expressions::precedences::property;
use crate::front_end::parser::{ParserContext, expressions::index};
use crate::front_end::types::ast::Ast;

#[inline]
pub fn index_precedence<'parser>(
    ctx: &mut ParserContext<'parser>,
) -> Result<Ast<'parser>, CompilationIssue> {
    let mut expr: Ast = property::property_precedence(ctx)?;

    while ctx.match_token(TokenType::LBracket)? {
        let span: Span = ctx.previous().span;

        expr = index::build_index(ctx, expr, span)?;
    }

    Ok(expr)
}
