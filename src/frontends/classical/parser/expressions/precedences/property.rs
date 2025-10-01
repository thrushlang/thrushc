use crate::{
    core::errors::standard::ThrushCompilerIssue,
    frontends::classical::{
        lexer::{span::Span, tokentype::TokenType},
        parser::{
            ParserContext,
            expressions::{precedences::lower, property},
        },
        types::{ast::Ast, parser::stmts::traits::TokenExtensions},
    },
};

pub fn property_precedence<'parser>(
    ctx: &mut ParserContext<'parser>,
) -> Result<Ast<'parser>, ThrushCompilerIssue> {
    let mut expression: Ast = lower::lower_precedence(ctx)?;

    if ctx.match_token(TokenType::Dot)? {
        let span: Span = ctx.previous().get_span();

        expression = property::build_property(ctx, (None, Some(expression.into()), span), span)?;
    }

    Ok(expression)
}
