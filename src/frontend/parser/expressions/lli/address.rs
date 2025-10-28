use crate::core::errors::standard::ThrushCompilerIssue;

use crate::frontend::lexer::{span::Span, token::Token, tokentype::TokenType};
use crate::frontend::parser::{ParserContext, expr};
use crate::frontend::types::{ast::Ast, parser::stmts::traits::TokenExtensions};
use crate::frontend::typesystem::types::Type;

pub fn build_address<'parser>(
    ctx: &mut ParserContext<'parser>,
) -> Result<Ast<'parser>, ThrushCompilerIssue> {
    let address_tk: &Token = ctx.advance()?;
    let address_span: Span = address_tk.get_span();

    let source: Ast = expr::build_expr(ctx)?;
    let expr_span: Span = source.get_span();

    let indexes: Vec<Ast> = self::build_address_indexes(ctx, expr_span)?;

    Ok(Ast::Address {
        source: source.into(),
        indexes,
        kind: Type::Addr,
        span: address_span,
    })
}

fn build_address_indexes<'parser>(
    ctx: &mut ParserContext<'parser>,
    span: Span,
) -> Result<Vec<Ast<'parser>>, ThrushCompilerIssue> {
    ctx.consume(
        TokenType::LBrace,
        "Syntax error".into(),
        "Expected '{'.".into(),
    )?;

    let mut indexes: Vec<Ast> = Vec::with_capacity(10);

    loop {
        if ctx.check(TokenType::RBrace) {
            break;
        }

        let index: Ast = expr::build_expr(ctx)?;

        indexes.push(index);

        if ctx.check(TokenType::RBrace) {
            break;
        } else {
            ctx.consume(
                TokenType::Comma,
                "Syntax error".into(),
                "Expected ','.".into(),
            )?;
        }
    }

    ctx.consume(
        TokenType::RBrace,
        "Syntax error".into(),
        "Expected '}'.".into(),
    )?;

    if indexes.is_empty() {
        return Err(ThrushCompilerIssue::Error(
            "Syntax error".into(),
            "At least one index was expected.".into(),
            None,
            span,
        ));
    }

    Ok(indexes)
}
