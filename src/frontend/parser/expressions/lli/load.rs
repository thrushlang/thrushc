use crate::{
    core::errors::standard::ThrushCompilerIssue,
    frontend::{
        lexer::{span::Span, token::Token, tokentype::TokenType},
        parser::{ParserContext, expr, typegen},
        types::{ast::Ast, parser::stmts::traits::TokenExtensions},
        typesystem::types::Type,
    },
};

pub fn build_load<'parser>(
    ctx: &mut ParserContext<'parser>,
) -> Result<Ast<'parser>, ThrushCompilerIssue> {
    let load_tk: &Token = ctx.consume(
        TokenType::Load,
        "Syntax error".into(),
        "Expected 'load' keyword.".into(),
    )?;

    let span: Span = load_tk.get_span();

    let load_type: Type = typegen::build_type(ctx)?;

    ctx.consume(
        TokenType::Comma,
        "Syntax error".into(),
        "Expected ','.".into(),
    )?;

    let source: Ast = expr::build_expr(ctx)?;

    Ok(Ast::Load {
        source: source.into(),
        kind: load_type,
        span,
    })
}
