use crate::{
    core::errors::standard::ThrushCompilerIssue,
    frontends::classical::{
        lexer::{span::Span, token::Token, tokentype::TokenType},
        parser::{ParserContext, expr, expressions::reference, typegen},
        types::{ast::Ast, parser::stmts::traits::TokenExtensions},
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

    if ctx.match_token(TokenType::Identifier)? {
        let identifier_tk: &Token = ctx.previous();
        let name: &str = identifier_tk.get_lexeme();

        let reference: Ast = reference::build_reference(ctx, name, span)?;

        ctx.consume(
            TokenType::Comma,
            "Syntax error".into(),
            "Expected ','.".into(),
        )?;

        let write_type: Type = typegen::build_type(ctx)?;

        let value: Ast = expr::build_expr(ctx)?;

        return Ok(Ast::Write {
            source: (Some((name, reference.into())), None, span),
            write_value: value.clone().into(),
            write_type,
            span,
        });
    }

    let expression: Ast = expr::build_expr(ctx)?;

    ctx.consume(
        TokenType::Comma,
        "Syntax error".into(),
        "Expected ','.".into(),
    )?;

    let write_type: Type = typegen::build_type(ctx)?;
    let value: Ast = expr::build_expr(ctx)?;

    Ok(Ast::Write {
        source: (None, Some(expression.into()), span),
        write_value: value.clone().into(),
        write_type,
        span,
    })
}
