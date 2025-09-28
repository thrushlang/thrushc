use crate::{
    core::errors::standard::ThrushCompilerIssue,
    frontends::classical::{
        lexer::{span::Span, token::Token, tokentype::TokenType},
        parser::{ParserContext, expr, expressions::reference, typegen},
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

    if ctx.check(TokenType::Identifier) {
        let identifier_tk: &Token = ctx.consume(
            TokenType::Identifier,
            "Syntax error".into(),
            "Expected 'identifier'.".into(),
        )?;

        let reference_name: &str = identifier_tk.get_lexeme();

        let reference: Ast = reference::build_reference(ctx, reference_name, span)?;

        return Ok(Ast::Load {
            source: (Some((reference_name, reference.into())), None),
            kind: load_type,
            span,
        });
    }

    let expression: Ast = expr::build_expr(ctx)?;

    Ok(Ast::Load {
        source: (None, Some(expression.into())),
        kind: load_type,
        span,
    })
}
