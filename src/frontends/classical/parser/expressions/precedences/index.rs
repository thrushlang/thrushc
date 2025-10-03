use crate::{
    core::errors::standard::ThrushCompilerIssue,
    frontends::classical::{
        lexer::{span::Span, tokentype::TokenType},
        parser::{
            ParserContext,
            expressions::{index, precedences::property},
        },
        types::ast::Ast,
    },
};

#[inline]
pub fn index_precedence<'parser>(
    ctx: &mut ParserContext<'parser>,
) -> Result<Ast<'parser>, ThrushCompilerIssue> {
    let mut expr: Ast = property::property_precedence(ctx)?;

    while ctx.match_token(TokenType::LBracket)? {
        let span: Span = ctx.previous().span;

        expr = index::build_index(ctx, expr, span)?;
    }

    Ok(expr)
}
