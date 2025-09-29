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
    let expression: Ast = property::property_precedence(ctx)?;

    if ctx.match_token(TokenType::LBracket)? {
        let span: Span = ctx.previous().span;
        let index: Ast = index::build_index(ctx, (None, Some(expression.into())), span)?;

        return Ok(index);
    }

    Ok(expression)
}
