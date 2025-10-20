use crate::{
    core::errors::standard::ThrushCompilerIssue,
    frontend::{
        lexer::{span::Span, token::Token, tokentype::TokenType},
        parser::{ParserContext, expr, typegen},
        types::ast::Ast,
        typesystem::types::Type,
    },
};

pub fn build_write<'parser>(
    ctx: &mut ParserContext<'parser>,
) -> Result<Ast<'parser>, ThrushCompilerIssue> {
    let write_tk: &Token = ctx.consume(
        TokenType::Write,
        "Syntax error".into(),
        "Expected 'write' keyword.".into(),
    )?;

    let span: Span = write_tk.span;

    let source: Ast = expr::build_expr(ctx)?;

    ctx.consume(
        TokenType::Comma,
        "Syntax error".into(),
        "Expected ','.".into(),
    )?;

    let write_type: Type = typegen::build_type(ctx)?;
    let value: Ast = expr::build_expr(ctx)?;

    Ok(Ast::Write {
        source: source.into(),
        write_value: value.clone().into(),
        write_type,
        span,
    })
}
